#!/usr/bin/env python3

from functools import reduce

from boto3 import client

from instance import InstanceService
from investigation_logger import CLAIRE


class SnapshotCreationService:
    ec2 = client("ec2")
    instance_service: InstanceService
    logger: callable

    def __init__(self, investigation_id: str):
        self.instance_service = InstanceService(investigation_id)
        self.logger = self.instance_service.logger

    def snapshot_volumes(self, investigation_id: str):
        instance = self.instance_service.get_instance(investigation_id)
        return list(
            map(
                lambda v: self.__snapshot(v["VolumeId"], investigation_id),
                self.instance_service.get_volumes(instance),
            ))

    def is_snapshot_complete(self, snapshot_ids: str):
        self.logger("Getting snapshot status for {}".format(snapshot_ids))
        return reduce(
            lambda is_ready, s: is_ready
            if s["State"] == "completed" else False,
            self.ec2.describe_snapshots(SnapshotIds=snapshot_ids)["Snapshots"],
            True,
        )

    def __snapshot(self, volume_id: str, investigation_id: str):
        tags = [{
            "Key": CLAIRE,
            "Value": "Investigating",
        }, {
            "Key": "InvestigationId",
            "Value": investigation_id,
        }]

        self.logger("Tagging volume {}".format(volume_id))
        self.ec2.create_tags(Resources=[volume_id], Tags=tags)
        self.logger("Tags created for volume {}".format(volume_id))

        self.logger("Creating snapshot for volume {}".format(volume_id))
        resp = self.ec2.create_snapshot(
            Description="Created by CLAIRE for investigation {}".format(
                investigation_id),
            VolumeId=volume_id,
            TagSpecifications=[{
                "ResourceType": "snapshot",
                "Tags": tags,
            }],
            DryRun=False)
        self.logger("Snapshot of volume {} complete {}".format(
            volume_id, resp))

        return resp["SnapshotId"]


def lambda_snapshot_handler(event: object, _):
    snapper = SnapshotCreationService(event["investigation_id"])
    event["snapshot_ids"] = snapper.snapshot_volumes(event["investigation_id"])
    event["is_ready"] = False

    return event


def lambda_snapshot_ready_handler(event, _):
    snapper = SnapshotCreationService(event["investigation_id"])
    event["is_ready"] = snapper.is_snapshot_complete(event["snapshot_ids"])

    return event
