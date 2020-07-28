#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, log_to_console, to_json
from get_instance import InstanceService
from snapshot_volumes import lambda_snapshot_handler, lambda_snapshot_ready_handler
from manage_volumes import move_volumes, MoveVolumesRequst, lambda_create_volumes
from run_command import lambda_is_command_complete
from sys import argv
from json import dumps
from time import sleep
from shlex import quote
from functools import reduce
from os import environ


class VolumeCaptureService:
    ec2 = client("ec2")
    instance_service: InstanceService
    logger: callable

    def __init__(self, investigation_id):
        self.instance_service = InstanceService(investigation_id)
        self.logger = self.instance_service.logger

    def capture_volumes(self, volumes: list, investigation_id: str,
                        bucket: str) -> dict:
        instance_id = self.instance_service.get_extractor_instance(
            investigation_id)["InstanceId"]
        ssm = client("ssm")
        self.logger(
            "Sending volume capture commands to {}".format(instance_id))

        upload = "sudo aws s3 sync /tmp/investigation 's3://{}/{}/'".format(
            environ["INVESTIGATION_BUCKET"], investigation_id)

        resp = ssm.send_command(
            DocumentName="AWS-RunShellScript",
            InstanceIds=[instance_id],
            Comment="Acquiring volumes for investigation {}".format(
                investigation_id),
            TimeoutSeconds=3600,
            Parameters={
                "commands": [
                    "sudo apt update",
                    "sudo apt install --no-install-recommends --yes awscli sleuthkit",
                ] + self.__capture_root_volume(volumes.pop(0)) + reduce(
                    lambda commands, vol: commands + self.__capture_volume(
                        vol), volumes, []) + [upload, upload]
            },
            OutputS3BucketName=environ["INVESTIGATION_BUCKET"],
            OutputS3KeyPrefix="{}/cmd/extract-volumes".format(
                investigation_id),
        )

        return {
            "running_command_instance": instance_id,
            "running_command_id": resp["Command"]["CommandId"],
        }

    def __capture_volume(self, vol: dict) -> list:
        return [
            "sudo tsk_gettimes {}1 \
                | mactime -d -y \
                    -p /tmp/investigation/artifacts/passwd \
                    -g /tmp/investigation/artifacts/group \
                | gzip > /tmp/investigation/timeline/{}.csv.gz".format(
                quote(vol["device"]), quote(vol["snapshot_id"]))
        ]

    def __capture_root_volume(self, vol: dict) -> list:
        return [
            "sudo mkdir -p /mnt/vol",
            "sudo rm -rf /tmp/investigation; mkdir -p /tmp/investigation/logs;  mkdir /tmp/investigation/artifacts; mkdir /tmp/investigation/timeline",
            "sudo mount -o ro {}1 /mnt/vol".format(quote(vol["device"])),
            "cd /mnt/vol/var/log; sudo bash -c 'find . -type f -exec grep -Iq . {} + -print | cpio -pdm /tmp/investigation/logs/'",
            "sudo gzip -r /tmp/investigation/logs/",
            "sudo last -f /mnt/vol/var/log/wtmp > /tmp/investigation/logs/wtmp # system power logs",
            "sudo last -f /mnt/vol/var/log/btmp > /tmp/investigation/logs/btmp # login attempts",
            "cd /mnt/vol/etc/; sudo bash -c 'cp {passwd,group,shadow,hostname,timezone,hosts,resolv.conf,*-release,sudoers,fstab,crontab} /tmp/investigation/artifacts/'",
            "sudo tsk_gettimes {} \
                | mactime -d -y \
                    -p /tmp/investigation/artifacts/passwd \
                    -g /tmp/investigation/artifacts/group \
                | gzip > /tmp/investigation/timeline/root-{}.csv.gz".format(
                quote(vol["device"]), vol["snapshot_id"]),
            "cd /; sudo umount /mnt/vol",
        ]


def lambda_capture_volumes(event, context):
    dc = VolumeCaptureService(event["investigation_id"])
    event["data"]["instance_from"] = event["data"]["instance_to"]
    event["data"]["instance_to"] = None
    return {
        **event,
        **dc.capture_volumes(
            event["volumes"],
            event["investigation_id"],
            environ["INVESTIGATION_BUCKET"],
        ),
        **{
            "is_ready": False
        },
    }


def main():
    log_to_console()
    try:
        ins = InstanceService(argv[1])
        event = {
            "investigation_id": argv[1],
            "instance_id": ins.get_instance(argv[1])["InstanceId"],
            "extractor_id": ins.get_instance(argv[1], "Worker")["InstanceId"]
        }

    except IndexError:
        print("Usage {} [investigation_id] [investigation_bucket]".format(
            argv[0]))
        return 1

    event = lambda_snapshot_handler(event, {})
    while not event["is_ready"]:
        sleep(5)
        event = lambda_snapshot_ready_handler(event, {})

    event = lambda_create_volumes(event, {})
    print(to_json(event))

    move_volumes(
        {
            "investigation_id":
            event["investigation_id"],
            "data":
            MoveVolumesRequst(
                event["investigation_id"],
                event["volumes"],
                None,
                event["extractor_id"],
            ).asdict(),
            "is_ready":
            False
        }, {})

    event = lambda_capture_volumes(event, {})
    while not event["is_ready"]:
        sleep(60)
        event = lambda_is_command_complete(event, {})

    print(to_json(event))


if __name__ == "__main__":
    main()
