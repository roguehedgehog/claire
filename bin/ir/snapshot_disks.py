#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, CLAIRE
from sys import argv
from json import dumps


class SnapshotCreationService:
    ec2 = client("ec2")
    logger: object

    def snapshot_volumes(self, investigation_id: str, from_console: bool):
        try:
            self.logger = get_logger(investigation_id)
            instance = self.__get_instance(investigation_id)
            print(instance)
            volumes = self.__get_volumes(instance)
            for volume in volumes:
                self.__snapshot(volume["VolumeId"], investigation_id)

        except ValueError as e:
            self.logger("Unexpected value: {}".format(e))
            return {"result": "FAIL", "investigation_id": investigation_id}

    

    def __get_volumes(self, instance: object):
        self.logger("Getting volumes")
        resp = self.ec2.describe_volumes(
            Filters=[{
                "Name": "attachment.instance-id",
                "Values": [instance["InstanceId"]],
            }])
        self.logger("Found {} volumes".format(len(resp["Volumes"])))

        return resp["Volumes"]

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
            Description="Created by CLARE for investigation {}".format(
                investigation_id),
            VolumeId=volume_id,
            TagSpecifications=[{
                "ResourceType": "snapshot",
                "Tags": tags,
            }],
            DryRun=False)
        self.logger("Snapshot of volume {} complete {}".format(
            volume_id, resp))


def lambda_handler(event: object, context: object):
    snapper = SnapshotCreationService()
    snapper.snapshot_volumes(event["investigation_id"], from_console=False)


def main():
    if len(argv) < 2:
        print("Usage: {} [investigation_id]".format(argv[0]))
        exit(1)

    snapper = SnapshotCreationService()
    snapper.snapshot_volumes(argv[1], from_console=True)


if __name__ == "__main__":
    main()
