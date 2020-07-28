#!/usr/bin/env python3

from boto3 import client
from botocore.exceptions import ClientError
from os import environ
from investigation_logger import CLAIRE, get_logger, to_json
from datetime import datetime
from sys import argv, stderr
from json import dumps


class InvestigationCreationService:
    ec2 = client("ec2")
    s3 = client("s3")
    logger = None

    def create_investigation_from_guardduty(self, event: object) -> object:
        self.logger = get_logger(None)
        (is_investigatable, issue) = self.__is_investigatable(event)
        if not is_investigatable:
            self.logger("Event will not be investigated {} because {}".format(
                event, issue))
            return {
                "investigation_id": None,
                "instance_id": None,
                "issue": issue
            }

        instance_id = event["detail"]["resource"]["instanceDetails"][
            "instanceId"]
        self.logger("Instance {} will be investigated.".format(instance_id))

        investigation_id = self.create_investigation(instance_id, event)

        return {
            "investigation_id": investigation_id,
            "instance_id": instance_id
        }

    def __is_investigatable(self, event: object) -> (bool, str):
        if event == {}:
            return (False, "The event does not contain any information")

        try:
            if event["detail"]["resource"]["resourceType"] != "Instance":
                return (False, "The alert in not for an instance")

            instance_id = event["detail"]["resource"]["instanceDetails"][
                "instanceId"]

            if instance_id == "":
                return (False, "The instance id is empty")

            instance = self.__get_instance(instance_id)
            if not instance:
                return (False,
                        "Instance {} cannot be found".format(instance_id))
            (
                is_under_investigaiton,
                investigation_id,
            ) = self.__is_under_investigation(instance)
            if is_under_investigaiton:
                return (
                    False,
                    "The instance {} is being investigated by investigation {}"
                    .format(instance_id, investigation_id))

            volume_attached_to_memory_device = [
                device["Ebs"]["VolumeId"]
                for device in instance["BlockDeviceMappings"]
                if device["DeviceName"] == "/dev/sdm"
                or device["DeviceName"] == "/dev/xvdm"
            ]

            if volume_attached_to_memory_device:
                return (
                    False,
                    "Volume {} is attached to xvdm which will be used to capture memory"
                    .format(volume_attached_to_memory_device))

            return (True, "")

        except KeyError as e:
            return (False, "Invalid request: {}".format(e))

        except ClientError as e:
            return (False, "AWS returned: {}".format(e))

    def create_investigation(self, instance_id: str, event: object) -> str:
        self.logger = get_logger(None)
        instance = self.__get_instance(instance_id)
        investigation_id = "{}_{}".format(datetime.now(),
                                          instance["InstanceId"])

        self.logger = get_logger(investigation_id)

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
        if resp["Reservations"] == []:
            return None

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

    def __is_under_investigation(self, instance: object) -> (bool, str):
        if CLAIRE in [tag["Key"] for tag in instance["Tags"]]:
            return (True, self.__get_investigation_id(instance))

        return (False, "")

    def __get_investigation_id(self, instance: object) -> str:
        for tag in instance["Tags"]:
            if tag["Key"] == "InvestigationId":
                return tag["Value"]


def lambda_handler(event, context):
    creator = InvestigationCreationService()
    return creator.create_investigation_from_guardduty(event)


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

    resp = InvestigationCreationService().create_investigation(
        argv[1], {"details": "Manually triggered"})

    print(to_json(resp))


if __name__ == "__main__":
    main()