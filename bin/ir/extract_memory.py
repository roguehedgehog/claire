#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console
from run_command import lambda_is_command_complete
from get_instance import InstanceService
from manage_volumes import move_volumes, MoveVolumesRequst
from sys import argv
from os import environ
from pathlib import Path
from time import sleep


class MemoryCaptureService:
    ec2 = client("ec2")
    instance_service: InstanceService
    logger: callable

    def __init__(self, investigation_id):
        self.instance_service = InstanceService(investigation_id)
        self.logger = self.instance_service.logger

    def prepare_volume(self, extractor_id: str, investigation_id: str):
        ssm = client("ssm")
        self.logger(
            "Preparing memory capture volume on {}".format(extractor_id))
        resp = ssm.send_command(
            DocumentName="AWS-RunShellScript",
            InstanceIds=[extractor_id],
            Comment="CLAIRE Prepare Volume for Memory Capture of {}".format(
                extractor_id),
            TimeoutSeconds=3600,
            Parameters={
                "commands": ["sudo memory_prepare_volume.sh /dev/xvdm"]
            },
            OutputS3BucketName=environ["INVESTIGATION_BUCKET"],
            OutputS3KeyPrefix="{}/cmd/prepare-memory-volume".format(
                investigation_id),
        )

        return resp["Command"]["CommandId"]

    def capture_memory(self, instance_id: str, volume_id: str,
                       investigation_id: str):
        ssm = client("ssm")
        self.logger(
            "Sending memory capture commands to {}".format(instance_id))
        resp = ssm.send_command(
            DocumentName="AWS-RunShellScript",
            InstanceIds=[instance_id],
            Comment="CLAIRE Memory Capture of {} to {}".format(
                instance_id,
                volume_id,
            ),
            TimeoutSeconds=3600,
            Parameters={
                "commands": [
                    "sudo mkdir -p /mnt/mem",
                    "sudo mount /dev/xvdm /mnt/mem",
                    "sudo /mnt/mem/avml --compress /mnt/mem/memory.lime.compressed",
                    "sudo umount /mnt/mem",
                ]
            },
        )

        return resp["Command"]["CommandId"]

    def upload_memory(self, investigation_id: str, instance_id: str,
                      bucket: str):
        ssm = client("ssm")
        self.logger("Sending memory upload commands to {}".format(instance_id))
        resp = ssm.send_command(
            DocumentName="AWS-RunShellScript",
            InstanceIds=[instance_id],
            Comment="Uploading memory for investigation {}".format(
                investigation_id),
            TimeoutSeconds=3600,
            Parameters={
                "commands": [
                    "sudo memory_analysis.sh /dev/xvdm 's3://{}/{}/memory/'".
                    format(
                        bucket,
                        investigation_id,
                    )
                ]
            },
            OutputS3BucketName=environ["INVESTIGATION_BUCKET"],
            OutputS3KeyPrefix="{}/cmd/upload-memory".format(investigation_id),
        )

        return resp["Command"]["CommandId"]


def lambda_prepare_memory_volume(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    event["running_command_instance"] = event["extractor_id"]
    event["running_command_id"] = mcs.prepare_volume(event["extractor_id"],
                                                     event["investigation_id"])
    event["is_ready"] = False

    return event


def lambda_capture_memory(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])

    event["running_command_instance"] = event["instance_id"]
    event["running_command_id"] = mcs.capture_memory(
        event["instance_id"],
        event["memory_volume_id"],
        event["investigation_id"],
    )

    event["move_volumes"] = MoveVolumesRequst(
        event["investigation_id"],
        [{
            "volume_id": event["memory_volume_id"],
            "device": "/dev/xvdm"
        }],
        event["instance_id"],
        event["extractor_id"],
    ).asdict()

    event["is_ready"] = False

    return event


def lambda_upload_memory(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    event["running_command_instance"] = event["extractor_id"]
    event["running_command_id"] = mcs.upload_memory(
        event["investigation_id"],
        event["extractor_id"],
        environ["INVESTIGATION_BUCKET"],
    )

    event["move_volumes"] = MoveVolumesRequst(
        event["investigation_id"],
        [{
            "volume_id": event["memory_volume_id"],
            "device": None
        }],
        event["extractor_id"],
        None,
    ).asdict()

    event["is_ready"] = False

    return event


def main():
    log_to_console()
    try:
        ins = InstanceService(argv[1])
        event = {
            "investigation_id": argv[1],
            "memory_volume_id": argv[2],
            "instance_id": ins.get_instance(argv[1])["InstanceId"],
            "extractor_id": ins.get_instance(argv[1], "Worker")["InstanceId"],
        }
        event["move_volumes"] = MoveVolumesRequst(
            event["investigation_id"],
            [{
                "volume_id": argv[2],
                "device": "/dev/xvdm"
            }],
            event["extractor_id"],
            event["instance_id"],
        )

        context = {}

        environ["INVESTIGATION_BUCKET"] = argv[3]

    except IndexError:
        print("Usage {} [investigation_id] [volume_id] [investigation_bucket]".
              format(argv[0]))

        return 1

    event = lambda_prepare_memory_volume(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_command_complete(event, context)

    event = move_volumes(event, context)
    event = lambda_capture_memory(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_command_complete(event, context)

    event = move_volumes(event, context)
    event = lambda_upload_memory(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_command_complete(event, context)

    move_volumes(event, context)


if __name__ == "__main__":
    main()
