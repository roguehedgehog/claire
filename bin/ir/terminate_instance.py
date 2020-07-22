#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, log_to_console
from get_instance import InstanceService
from sys import argv
from os import environ


def terminate_instance(investigation_id: str, instance_id: str):
    ec2 = client("ec2")
    logger = get_logger(investigation_id)
    logger("Terminating instance {}".format(instance_id))

    ec2.terminate_instances(InstanceIds=[instance_id])
    logger("Terminate request succeeded")


def main():
    try:
        log_to_console()
        terminate_instance(argv[1], argv[2])
    except IndexError:
        print("Usage {} [investigation_id] [instance_id]".format(argv[0]))


if __name__ == "__main__":
    main()