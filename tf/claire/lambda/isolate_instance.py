#!/usr/bin/env python3

from os import environ

from boto3 import client

from instance import InstanceService
from investigation_logger import get_logger


def isolate(investigation_id: str, security_group):
    ec2 = client("ec2")
    logger = get_logger(investigation_id)
    instance = InstanceService(investigation_id).get_instance(investigation_id)

    logger("Changing security group to {}".format(security_group))
    ec2.modify_instance_attribute(
        InstanceId=instance["InstanceId"],
        Groups=[security_group],
    )
    logger("Security group updated successfully")


def lambda_handler(event: object, _):
    isolate(
        event["investigation_id"],
        environ["LOCKED_DOWN_SECURITY_GROUP"],
    )
