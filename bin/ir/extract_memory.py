#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console
from run_command import lambda_is_command_complete
from get_instance import InstanceService
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

    def prepare_volume(self, extractor_id: str):
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
                "commands": [
                    "mkdir -p /mnt/mem",
                    "mkfs -t ext4 /dev/xvdm",
                    "mount /dev/xvdm /mnt/mem",
                    "mkdir /mnt/mem/memory",
                    "cd /mnt/mem/memory",
                    "wget https://github.com/microsoft/avml/releases/download/v0.2.0/avml",
                    "chmod +x avml",
                    "cd /",
                    "umount /mnt/mem",
                ]
            })

        return resp["Command"]["CommandId"]

    def detach_volume(self, instance_id: str, volume_id: str):
        self.logger("Detaching volume {} from {}".format(
            volume_id,
            instance_id,
        ))
        resp = self.ec2.detach_volume(
            VolumeId=volume_id,
            InstanceId=instance_id,
        )
        self.logger("Detach volume request complete {}".format(resp))

    def attach_volume(self, instance_id: str, volume_id: str):
        self.logger("Attaching volume {} to instance {}".format(
            volume_id, instance_id))
        resp = self.ec2.attach_volume(
            InstanceId=instance_id,
            VolumeId=volume_id,
            Device="/dev/sdm",
        )
        self.logger("Attach request complete {}".format(resp))

    def is_detached(self, volume_id: str) -> bool:
        return self.__get_volume_state(volume_id) == "available"

    def is_attached(self, volume_id: str) -> bool:
        return self.__get_volume_state(volume_id) == "in-use"

    def __get_volume_state(self, volume_id: str) -> str:
        self.logger("Getting volume {} status".format(volume_id))
        state = self.ec2.describe_volumes(
            VolumeIds=[volume_id])["Volumes"][0]["State"]
        self.logger("Volume {} is {}".format(volume_id, state))

        return state

    def capture_memory(self, instance_id: str, volume_id: str):
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
                    "cd /mnt/mem/memory",
                    "sudo ./avml --compress memory.lime",
                    "cd /",
                    "sudo umount /mnt/mem",
                ]
            })

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
                    "sudo apt update",
                    "sudo apt install -y awscli",
                    "sudo mkdir -p /mnt/mem",
                    "sudo mount -o ro /dev/xvdm /mnt/mem",
                    "cd /mnt/mem/memory",
                    "aws s3 cp memory.lime 's3://{}/{}/'".format(
                        bucket,
                        investigation_id,
                    ),
                    "cd /",
                    "sudo umount /mnt/mem",
                ]
            })

        return resp["Command"]["CommandId"]

    def delete_volume(self, volume_id: str):
        self.logger("Deleting volume {}".format(volume_id))
        resp = self.ec2.delete_volume(VolumeId=volume_id)
        self.logger("Delete volume request complete {}".format(resp))


def lambda_prepare_memory_volume(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    event["running_command_instance"] = event["extractor_id"]
    event["running_command_id"] = mcs.prepare_volume(event["extractor_id"])
    event["is_ready"] = False

    return event


def lambda_detach_memory_volume(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    mcs.detach_volume(event["memory_vol_detach_from"],
                      event["memory_volume_id"])

    event["is_ready"] = False
    return event


def lambda_is_memory_volume_detached(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    event["is_ready"] = mcs.is_detached(event["memory_volume_id"])

    return event


def lambda_attach_memory_volume(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    mcs.attach_volume(event["memory_vol_attach_to"], event["memory_volume_id"])

    event["is_ready"] = False

    return event


def lambda_is_memory_volume_attached(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    event["is_ready"] = mcs.is_attached(event["memory_volume_id"])

    return event


def lambda_delete_memory_volume(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])
    mcs.delete_volume(event["memory_volume_id"])

    return event


def lambda_capture_memory(event: object, context: object):
    mcs = MemoryCaptureService(event["investigation_id"])

    event["running_command_instance"] = event["instance_id"]
    event["running_command_id"] = mcs.capture_memory(
        event["instance_id"],
        event["memory_volume_id"],
    )

    event["memory_vol_detach_from"] = event["instance_id"]
    event["memory_vol_attach_to"] = event["extractor_id"]
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
    event["memory_vol_detach_from"] = event["extractor_id"]
    event["memory_vol_attach_to"] = None
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
        event["memory_vol_detach_from"] = event["extractor_id"]
        event["memory_vol_attach_to"] = event["instance_id"]

        context = {}

        environ["INVESTIGATION_BUCKET"] = argv[3]

    except IndexError:
        print("Usage {} [investigation_id] [volume_id] [investigation_bucket]".
              format(argv[0]))

    event = lambda_prepare_memory_volume(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_command_complete(event, context)

    event = move_volume(event, context)
    event = lambda_capture_memory(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_command_complete(event, context)

    event = move_volume(event, context)
    event = lambda_upload_memory(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_command_complete(event, context)

    move_volume(event, context)


def move_volume(event, context):
    event = lambda_detach_memory_volume(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_memory_volume_detached(event, context)

    if event["memory_vol_attach_to"] is None:
        lambda_delete_memory_volume(event, context)
        return

    event = lambda_attach_memory_volume(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_memory_volume_attached(event, context)

    return event


if __name__ == "__main__":
    main()
