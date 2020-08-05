#!/usr/bin/env python3

from datetime import datetime
from json import dumps
from os import environ

from boto3 import client
from botocore.exceptions import ClientError

from investigation_logger import CLAIRE, get_logger


class InvestigationCreationService:
    ec2 = client("ec2")
    s3 = client("s3")
    logger = None

    def create_investigation_from_alert(self, event: object,
                                        execution_id: str) -> object:
        self.logger = get_logger(None)
        (is_investigatable, issue, err) = self.__is_investigatable(event)
        if not is_investigatable:
            self.logger("Event will not be investigated {} because {}".format(
                event, issue))
            return {
                "investigation_id": None,
                "instance_id": None,
                "execution_arn": execution_id,
                "issue": issue,
                "err": err
            }

        instance_id = event["detail"]["resource"]["instanceDetails"][
            "instanceId"]
        self.logger("Instance {} will be investigated.".format(instance_id))

        investigation_id = self.create_investigation(instance_id, event,
                                                     execution_id)

        return {
            "investigation_id": investigation_id,
            "instance_id": instance_id,
            "execution_arn": execution_id,
            "err": "",
        }

    def __is_investigatable(self, event: object) -> (bool, str):
        if event == {}:
            return (False, "The event does not contain any information",
                    "InvalidInput")

        try:
            if event["detail"]["resource"]["resourceType"] != "Instance":
                return (False, "The alert in not for an instance",
                        "InvalidInput")

            instance_id = event["detail"]["resource"]["instanceDetails"][
                "instanceId"]

            if instance_id == "":
                return (False, "The instance id is empty", "InvalidInput")

            instance = self.__get_instance(instance_id)
            if not instance:
                return (False,
                        "Instance {} cannot be found".format(instance_id),
                        "InstanceNotFound")
            (
                is_under_investigaiton,
                investigation_id,
            ) = self.__is_under_investigation(instance)
            if is_under_investigaiton:
                return (
                    False,
                    "The instance {} is being investigated by investigation {}"
                    .format(instance_id,
                            investigation_id), "InvestigationInProgress")

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
                    .format(volume_attached_to_memory_device),
                    "MemoryCaptureDeviceInUse")

            return (True, "", "")

        except KeyError as e:
            return (False, "Invalid request: {}".format(e), "InvalidInput")

        except ClientError as e:
            return (False, "AWS returned: {}".format(e), "AWSClientError")

    def create_investigation(self, instance_id: str, event: object,
                             execution_id: str) -> str:
        self.logger = get_logger(None)
        instance = self.__get_instance(instance_id)
        investigation_id = "{}_{}".format(datetime.now(),
                                          instance["InstanceId"])

        self.logger = get_logger(investigation_id)

        self.__put(investigation_id, "alert.json", event, execution_id)
        self.__put(investigation_id, "instance.json", instance, execution_id)

        self.__tag(
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

    def __put(self, investigation_id: str, name: str, details: object,
              execution_id: str):
        self.logger("Putting {}".format(name))
        self.s3.put_object(
            Bucket=environ["INVESTIGATION_BUCKET"],
            Key="{}/{}".format(investigation_id, name),
            Body=dumps(details, indent=2, skipkeys=True, default=str),
            Tagging="CLAIRE_EXEC={}".format(execution_id),
        )
        self.logger("Putting {} complete".format(name))

    def __tag(self, instance_id: str, tags: list):
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

        return None


def lambda_handler(event, _):
    creator = InvestigationCreationService()
    try:
        payload = event["payload"]
        execution_id = event["execution_arn"]
    except KeyError:
        payload = event
        execution_id = None

    return creator.create_investigation_from_alert(payload, execution_id)
