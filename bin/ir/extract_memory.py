#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console
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

    def detech_volume(self, instance_id: str, volume_id: str):
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

    def get_command_status(self, instance_id: str, command_id: str):
        ssm = client("ssm")
        self.logger("Checking command {} status".format(command_id))
        resp = ssm.get_command_invocation(
            InstanceId=instance_id,
            CommandId=command_id,
        )
        self.logger("Command {} status returned {}".format(command_id, resp))

        return resp["Status"]

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
                    "sudo apt install awscli",
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


def main():
    try:
        investigation_id = argv[1]
        volume_id = argv[2]

        log_to_console()
        mcs = MemoryCaptureService(argv[1])
        ins = InstanceService(argv[1])

        instance_id = ins.get_instance(investigation_id)["InstanceId"]
        extractor_id = ins.get_instance(
            investigation_id,
            "Worker",
        )["InstanceId"]

        print(to_json(mcs.detech_volume(extractor_id, volume_id)))
        while mcs.is_detached(volume_id) == False:
            sleep(1)
            print(".", flush=True, end="")

        print("deteched, attaching to suspicious instance", flush=True, end="")

        mcs.attach_volume(instance_id, volume_id)
        while mcs.is_attached(volume_id) == False:
            sleep(1)
            print(".", flush=True, end="")

        print("attached")

        command_id = mcs.capture_memory(instance_id, volume_id)
        sleep(1)
        while mcs.get_command_status(instance_id, command_id) in [
                "Pending", "Delayed", "InProgress"
        ]:
            sleep(1)
            print(".", flush=True, end="")

        if "Success" != mcs.get_command_status(instance_id, command_id):
            raise Exception("Memory capture failed")

        print("capture complete")

        mcs.detech_volume(instance_id, volume_id)
        while mcs.is_detached(volume_id) == False:
            sleep(1)
            print(".", flush=True, end="")

        print("deteched, attaching to extractor instance", flush=True, end="")
        mcs.attach_volume(extractor_id, volume_id)
        while mcs.is_attached(volume_id) == False:
            sleep(1)
            print(".", flush=True, end="")

        command_id = mcs.upload_memory(
            investigation_id,
            extractor_id,
            "wild-pumpkin-investigations",
        )

        sleep(1)
        while mcs.get_command_status(extractor_id, command_id) in [
                "Pending", "Delayed", "InProgress"
        ]:
            sleep(1)
            print(".", flush=True, end="")

        if "Success" != mcs.get_command_status(extractor_id, command_id):
            raise Exception("Memory upload failed")

        print("upload complete, deleting volume")

        mcs.detech_volume(extractor_id, volume_id)
        while mcs.is_detached(volume_id) == False:
            sleep(1)
            print(".", flush=True, end="")

        mcs.delete_volume(volume_id)

    except IndexError:
        print("Usage {} [investigation_id] [volume_id]".format(argv[0]))


if __name__ == "__main__":
    main()
