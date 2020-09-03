#!/usr/bin/env python3

from boto3 import client

from investigation_logger import CLAIRE, get_logger


class InstanceService:
    ec2 = client("ec2")
    logger = None

    def __init__(self, investigation_id):
        self.logger = get_logger(investigation_id)

    def get_instance(self,
                     investigation_id: str,
                     action="Investigating") -> object:
        self.logger("Getting instance")
        resp = self.ec2.describe_instances(Filters=[
            {
                "Name": "instance-state-name",
                "Values": ["running", "stopped"],
            },
            {
                "Name": "tag:{}".format(CLAIRE),
                "Values": [action],
            },
            {
                'Name': 'tag:InvestigationId',
                'Values': [investigation_id]
            },
        ])
        if resp["Reservations"] == []:
            raise ValueError(
                "Cannot find {} instance for investigation {}".format(
                    action, investigation_id))
        instance = resp["Reservations"][0]["Instances"][0]
        self.logger("Found instance {}".format(instance["InstanceId"]))

        return instance

    def get_extractor_instance(self, investigation_id: str):
        return self.get_instance(investigation_id, "Worker")

    def get_memory_size(self, instance: object) -> int:
        self.logger("Getting memory for instance {}".format(
            instance["InstanceId"]))
        resp = self.ec2.describe_instance_types(
            InstanceTypes=[instance["InstanceType"]])

        return resp["InstanceTypes"][0]["MemoryInfo"]["SizeInMiB"]
