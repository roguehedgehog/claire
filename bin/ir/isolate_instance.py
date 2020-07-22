#!/usr/bin/env python3

from investigation_logger import get_logger, log_to_console
from get_instance import InstanceService
from boto3 import client
from sys import argv
from os import environ


class IsolateInstanceService:
    ec2 = client("ec2")
    logger: callable

    def isolate(self, investigation_id: str, security_group):
        self.logger = get_logger(investigation_id)
        instance = InstanceService(investigation_id).get_instance(
            investigation_id)

        self.logger("Changing security group to {}".format(security_group))
        self.ec2.modify_instance_attribute(
            InstanceId=instance["InstanceId"],
            Groups=[security_group],
        )
        self.logger("Security group updated successfully")


def lambda_handler(event: object, context: object):
    IsolateInstanceService().isolate(
        event["investigation_id"],
        environ["LOCKED_DOWN_SECURITY_GROUP"],
    )


def main():
    try:
        log_to_console()
        IsolateInstanceService().isolate(argv[1], argv[2])
    except IndexError:
        print("Usage: {} [investigation_id] [security_group_id]".format(
            argv[0]))


if __name__ == "__main__":
    main()