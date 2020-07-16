#!/usr/bin/env python3

from boto3 import client
from os import environ
from investigation_logger import CLAIRE, get_logger
from datetime import datetime
from sys import argv, stderr
from json import dumps


class InvestigationCreationService:
    ec2 = client("ec2")
    s3 = client("s3")
    logger = None

    def is_investigatable(self, event: object) -> bool:
        return "resourceType" in event["detail"]["resource"] and \
            event["detail"]["resource"]["resourceType"] == "Instance"

    def create_investigation(self, instance_id: str, event: object) -> str:
        self.logger = get_logger(None, __name__ == "__main__")
        instance = self.__get_instance(instance_id)
        if self.__is_under_investigation(instance):
            self.logger(
                "Instance {} is already under investigation: {}".format(
                    instance_id,
                    self.__get_investigation_id(instance),
                ))
            return

        investigation_id = "{}_{}".format(datetime.now(),
                                          instance["InstanceId"])

        self.logger = get_logger(investigation_id, __name__ == "__main__")

        self.__put(investigation_id, "alert.json", event)
        self.__put(investigation_id, "instance.json", instance)

        self.__tag(
            investigation_id,
            instance_id,
            [{
                "Key": CLAIRE,
                "Value": "Investigating"
            }, {
                "Key": "InvestigationId",
                "Value": investigation_id
            }],
        )

        return investigation_id

    def __get_instance(self, instance_id: str) -> object:
        self.logger("Getting instance details for {}".format(instance_id))
        resp = self.ec2.describe_instances(InstanceIds=[instance_id])
        instance = resp["Reservations"][0]["Instances"][0]
        self.logger("Instance {} found".format(instance_id))

        return instance

    def __put(self, investigation_id: str, name: str, details: object):
        self.logger("Putting {}".format(name))
        self.s3.put_object(
            Bucket=environ["INVESTIGATION_BUCKET"],
            Key="{}/{}".format(investigation_id, name),
            Body=dumps(details, indent=2, skipkeys=True, default=str),
        )
        self.logger("Putting {} complete".format(name))

    def __tag(self, investigation_id: str, instance_id: str, tags: list):
        self.logger("Tagging suspicious instance")
        self.ec2.create_tags(Resources=[instance_id], Tags=tags)
        self.logger("Tagging complete")

    def __is_under_investigation(self, instance: object) -> bool:
        return CLAIRE in [tag["Key"] for tag in instance["Tags"]]

    def __get_investigation_id(self, instance: object) -> str:
        for tag in instance["Tags"]:
            if tag["Key"] == "InvestigationId":
                return tag["Value"]


def lambda_handler(event, context):
    creator = InvestigationCreationService()
    creator.logger("Received event {}".format(event["id"]))
    if not creator.is_investigatable(event):
        creator.logger("Event {} {} will not be investigated".format(
            event["source"],
            event["id"],
        ))
        return {"investigation_id": None, "instance_id": None}

    creator.logger("Event {} {} can be investigated.".format(
        event["source"],
        event["id"],
    ))

    instance_id = event["detail"]["resource"]["instanceDetails"]["instanceId"]
    investigation_id = creator.create_investigation(instance_id, event)

    return {"investigation_id": investigation_id, "instance_id": instance_id}


def main():
    errors = []
    if len(argv) < 2:
        errors.append(
            "You must provide an instance id to create an investigation, {} [instance id]"
            .format(argv[0]))

    if "INVESTIGATION_BUCKET" not in environ:
        errors.append(
            "INVESTIGATION_BUCKET must be set, export INVESTIGATION_BUCKET=[bucket]"
        )

    if len(errors):
        print(errors)
        exit(1)

    InvestigationCreationService().create_investigation(
        argv[1], {"details": "Manually triggered"})


if __name__ == "__main__":
    main()