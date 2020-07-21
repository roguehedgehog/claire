#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console
from sys import argv


class GetInstanceService:
    ec2 = client("ec2")
    logger = None

    def get_instance(self, investigation_id: str) -> object:
        self.logger = get_logger(investigation_id)
        self.logger("Getting instance")
        resp = self.ec2.describe_instances(
            Filters=[{
                'Name': 'tag:InvestigationId',
                'Values': [investigation_id]
            }])
        if resp["Reservations"] == []:
            raise ValueError(
                "Cannot find instance for investigation {}".format(
                    investigation_id))
        instance = resp["Reservations"][0]["Instances"][0]
        self.logger("Found instance {}".format(instance["InstanceId"]))

        return instance


def main():
    try:
        log_to_console()
        print(to_json(GetInstanceService().get_instance(argv[1])))
    except IndexError:
        print("Usage {} [investigation_id] [...]".format(argv[0]))


if __name__ == "__main__":
    main()
