#!/usr/bin/env python3

from boto3 import client

from investigation_logger import get_logger


def terminate_instance(investigation_id: str, instance_id: str):
    ec2 = client("ec2")
    logger = get_logger(investigation_id)
    logger("Terminating instance {}".format(instance_id))

    ec2.terminate_instances(InstanceIds=[instance_id])
    logger("Terminate request succeeded")
