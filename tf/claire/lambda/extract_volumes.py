#!/usr/bin/env python3

from os import environ
from shlex import quote

from boto3 import client

from instance import InstanceService


def capture_volumes(volumes: list, investigation_id: str, bucket: str) -> dict:
    ins = InstanceService(investigation_id)
    instance_id = ins.get_extractor_instance(investigation_id)["InstanceId"]
    ssm = client("ssm")
    ins.logger("Sending volume capture commands to {}".format(instance_id))

    resp = ssm.send_command(
        DocumentName="AWS-RunShellScript",
        InstanceIds=[instance_id],
        Comment="Acquiring volumes for investigation {}".format(
            investigation_id),
        TimeoutSeconds=3600,
        Parameters={
            "commands": get_capture_commands(investigation_id, volumes)
        },
        OutputS3BucketName=bucket,
        OutputS3KeyPrefix="{}/cmd/capture-volumes".format(investigation_id),
    )

    return {
        "running_command_instance": instance_id,
        "running_command_id": resp["Command"]["CommandId"],
    }


def get_capture_commands(investigation_id: str, volumes: list) -> list:
    root_vol = volumes.pop(0)
    capture_root = "sudo volume_root_capture.sh {} {}".format(
        quote(root_vol["volume_id"]),
        quote(root_vol["snapshot_id"]),
    )
    capture_others = map(
        lambda vol: "sudo volume_timeline.sh {} {}".format(
            quote(vol["volume_id"]), vol["snapshot_id"]), volumes)

    upload = "sudo aws s3 sync /home/ubuntu/investigation 's3://{}/{}/'".format(
        environ["INVESTIGATION_BUCKET"], investigation_id)

    return [capture_root] + list(capture_others) + [upload, upload]


def lambda_capture_volumes(event, _):
    event["data"]["instance_from"] = event["data"]["instance_to"]
    event["data"]["instance_to"] = None
    return {
        **event,
        **capture_volumes(
            event["volumes"],
            event["investigation_id"],
            environ["INVESTIGATION_BUCKET"],
        ),
        **{
            "is_ready": False
        },
    }
