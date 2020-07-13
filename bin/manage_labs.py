#!/usr/bin/env python3

from boto3 import client
from sys import argv
from functools import reduce
from time import sleep
from datetime import datetime
import json

ec2 = client("ec2")


def get_instance_details():
    resp = ec2.describe_instances(Filters=[{
        'Name': 'tag:Lab',
        'Values': ['vuln_app'],
    }])

    instances = (instance for res in resp["Reservations"]
                 for instance in res["Instances"])

    return list(
        map(
            lambda instance: dict({
                "id":
                instance["InstanceId"],
                "state":
                instance["State"],
                "private_ip":
                instance["PrivateIpAddress"],
                "public_ip":
                instance["PublicIpAddress"]
                if "PublicIpAddress" in instance else "",
            }), instances))


def poll_instances(expected_status: str):
    while True:
        details = get_instance_details()
        all_ready = reduce(
            lambda acc, instance: acc
            if instance['state']["Name"] == expected_status else False,
            details,
            True,
        )

        print(
            "{} - {}".format(datetime.now().strftime("%H:%M:%S"), details),
            end="\r",
            flush=True,
        )
        if all_ready is True:
            print("\n")
            break

        sleep(1)


def main():
    details = get_instance_details()
    if details == []:
        print("No labs found")
        return

    if len(argv) < 2:
        print("Lab Status")
        print(json.dumps(*details, indent=2))
        return

    instanceIds = list(map(lambda instance: instance["id"], details))
    if "start" == argv[1]:
        print("Starting labs")
        ec2.start_instances(InstanceIds=instanceIds)
        poll_instances("running")
    elif "stop" == argv[1]:
        print("Stopping labs")
        ec2.stop_instances(InstanceIds=instanceIds)
        poll_instances("stopped")
    else:
        print("Unknown command '{}' valid commands are start|stop".format(
            argv[1]))


if __name__ == "__main__":
    main()