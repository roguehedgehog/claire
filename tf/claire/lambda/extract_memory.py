#!/usr/bin/env python3

from os import environ

from boto3 import client

from investigation_logger import get_logger
from manage_volumes import MoveVolumesRequst


def prepare_volume(extractor_id: str, investigation_id: str):
    ssm = client("ssm")
    get_logger(investigation_id)(
        "Preparing memory capture volume on {}".format(extractor_id))
    resp = ssm.send_command(
        DocumentName="AWS-RunShellScript",
        InstanceIds=[extractor_id],
        Comment="CLAIRE Prepare Volume for Memory Capture of {}".format(
            extractor_id),
        TimeoutSeconds=3600,
        Parameters={"commands": ["sudo memory_prepare_volume.sh /dev/xvdm"]},
        OutputS3BucketName=environ["INVESTIGATION_BUCKET"],
        OutputS3KeyPrefix="{}/cmd/prepare-memory-volume".format(
            investigation_id),
    )

    return resp["Command"]["CommandId"]


def capture_memory(instance_id: str, volume_id: str, investigation_id: str):
    ssm = client("ssm")
    get_logger(investigation_id)(
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


def run_memory_analysis(investigation_id: str, instance_id: str, bucket: str):
    ssm = client("ssm")
    get_logger(investigation_id)(
        "Sending memory upload commands to {}".format(instance_id))
    resp = ssm.send_command(
        DocumentName="AWS-RunShellScript",
        InstanceIds=[instance_id],
        Comment="Uploading memory for investigation {}".format(
            investigation_id),
        TimeoutSeconds=3600,
        Parameters={
            "commands": [
                "sudo memory_analysis.sh /dev/xvdm 's3://{}/{}'".format(
                    bucket,
                    investigation_id,
                )
            ]
        },
        OutputS3BucketName=environ["INVESTIGATION_BUCKET"],
        OutputS3KeyPrefix="{}/cmd/memory-analysis".format(investigation_id),
    )

    return resp["Command"]["CommandId"]


def lambda_prepare_memory_volume(event: object, _):
    event["running_command_instance"] = event["extractor_id"]
    event["running_command_id"] = prepare_volume(event["extractor_id"],
                                                 event["investigation_id"])
    event["is_ready"] = False

    return event


def lambda_capture_memory(event: object, _):
    event["running_command_instance"] = event["instance_id"]
    event["running_command_id"] = capture_memory(event["instance_id"],
                                                 event["memory_volume_id"],
                                                 event["investigation_id"])

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


def lambda_memory_analysis(event: object, _):
    event["running_command_instance"] = event["extractor_id"]
    event["running_command_id"] = run_memory_analysis(
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
