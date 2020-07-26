#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console
from get_instance import InstanceService
from sys import argv
from os import environ
from pathlib import Path
from time import sleep
from functools import reduce
from dataclasses import dataclass, asdict


@dataclass
class MoveVolumesRequst:
    investigation_id: str
    volumes: list
    instance_from: str
    instance_to: str

    def asdict(self):
        return asdict(self)


class ManageVolumesService:
    ec2 = client("ec2")
    logger: callable

    def __init__(self, investigation_id):
        self.logger = get_logger(investigation_id)

    def create_volumes(self, snapshot_ids: list, az: str) -> list:
        volume_ids = []
        mount_letter = "f"
        for snapshot in snapshot_ids:
            self.logger("Creating volume from {}".format(snapshot))
            resp = self.ec2.create_volume(AvailabilityZone=az,
                                          SnapshotId=snapshot)
            volume = {
                "volume_id": resp["VolumeId"],
                "device": "/dev/xvd{}".format(mount_letter),
                "snapshot_id": snapshot,
            }
            self.logger("Create volume request successful {}".format(volume))

            volume_ids.append(volume)
            mount_letter = chr(1 + ord(mount_letter))

        return volume_ids

    def detach_volumes(self, instance_id: str, volumes: list):
        for v in volumes:
            self.logger("Detaching volume {} from {}".format(
                v["volume_id"],
                instance_id,
            ))
            resp = self.ec2.detach_volume(
                VolumeId=v["volume_id"],
                InstanceId=instance_id,
            )
            self.logger("Detach volume request complete {}".format(resp))

    def attach_volumes(self, instance_id: str, volumes: list):
        for v in volumes:
            self.logger("Attaching volume {} to instance {}".format(
                v["volume_id"], instance_id))
            resp = self.ec2.attach_volume(
                InstanceId=instance_id,
                VolumeId=v["volume_id"],
                Device=v["device"],
            )
            self.logger("Attach request complete {}".format(resp))

    def is_status(self, volumes: list, status: str) -> bool:
        return reduce(
            lambda is_ready, v: is_ready
            if self.__get_volume_state(v["volume_id"]) == status else False,
            volumes,
            True,
        )

    def __get_volume_state(self, volume_id: str) -> str:
        self.logger("Getting volume {} status".format(volume_id))
        state = self.ec2.describe_volumes(
            VolumeIds=[volume_id])["Volumes"][0]["State"]
        self.logger("Volume {} is {}".format(volume_id, state))

        return state

    def destroy_volumes(self, volumes: list):
        for v in volumes:
            self.logger("Deleting volume {}".format(v["volume_id"]))
            resp = self.ec2.delete_volume(VolumeId=v["volume_id"])
            self.logger("Delete volume request complete {}".format(resp))


def lambda_create_volumes(event: object, context: object):
    id = event["investigation_id"]
    mvs = ManageVolumesService(id)
    extractor = InstanceService(id).get_extractor_instance(id)

    event["volumes"] = mvs.create_volumes(
        event["snapshot_ids"], extractor["Placement"]["AvailabilityZone"])
    event["data"] = {
        "investigation_id": event["investigation_id"],
        "volumes": event["volumes"],
        "instance_from": None,
        "instance_to": extractor["InstanceId"]
    }
    event["is_ready"] = False

    return event


def lambda_detach_volumes(event: object, context: object):
    mvs = ManageVolumesService(event["data"]["investigation_id"])
    mvs.detach_volumes(event["data"]["instance_from"],
                       event["data"]["volumes"])

    event["is_ready"] = False
    return event


def lambda_is_detach_complete(event: object, context: object):
    mvs = ManageVolumesService(event["data"]["investigation_id"])
    event["is_ready"] = mvs.is_status(event["data"]["volumes"], "available")

    return event


def lambda_attach_volumes(event: object, context: object):
    mvs = ManageVolumesService(event["data"]["investigation_id"])
    mvs.attach_volumes(event["data"]["instance_to"], event["data"]["volumes"])

    event["is_ready"] = False

    return event


def lambda_is_attach_complete(event: object, context: object):
    mvs = ManageVolumesService(event["data"]["investigation_id"])
    event["is_ready"] = mvs.is_status(event["data"]["volumes"], "in-use")

    return event


def lambda_destroy_volumes(event: object, context: object):
    mvs = ManageVolumesService(event["data"]["investigation_id"])
    mvs.destroy_volumes(event["data"]["volumes"])

    return event


def move_volumes(event, context):
    if event["data"]["instance_from"]:
        event = lambda_detach_volumes(event, context)

    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_detach_complete(event, context)

    if event["data"]["instance_to"] is None:
        lambda_destroy_volumes(event, context)
        return

    event = lambda_attach_volumes(event, context)
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_attach_complete(event, context)

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
        event["data"] = MoveVolumesRequst(
            argv[1],
            [{
                "volume_id": argv[2],
                "device": "/dev/xvdm"
            }],
            event["extractor_id"],
            event["instance_id"],
        ).asdict()

    except IndexError:
        print("Usage {} [investigation_id] [volume_id]".format(argv[0]))
        return 1

    move_volumes(event, {})


if __name__ == "__main__":
    main()