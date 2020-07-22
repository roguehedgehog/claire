#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json
from get_instance import InstanceService
from sys import argv
from os import environ
from pathlib import Path
from time import sleep

UBUNTU_2004_64_HVM = "ami-0127d62154efde733"


def create_extractor_instance(investagtion_id: str, security_group: str):
    ec2 = client("ec2")
    instance_service = InstanceService(investagtion_id)
    instance = instance_service.get_instance(investagtion_id)
    volumes = instance_service.get_volumes(instance)
    memory_size = instance_service.get_memory_size(instance)
    return ec2.run_instances(
        ImageId=UBUNTU_2004_64_HVM,
        InstanceType="t2.small",
        MinCount=1,
        MaxCount=1,
        InstanceInitiatedShutdownBehavior="terminate",
        SubnetId=instance["NetworkInterfaces"][0]["SubnetId"],
        SecurityGroupIds=[security_group],
        KeyName="vuln-app-key",
        BlockDeviceMappings=[{
            "DeviceName": "/dev/sdm",
            "Ebs": {
                "DeleteOnTermination": True,
                "VolumeSize": 8 + max(volumes, key=lambda v: v["Size"])["Size"]
            }
        }, {
            "DeviceName": "/dev/sda2",
            "Ebs": {
                "DeleteOnTermination": False,
                "VolumeSize": round((memory_size / 1000) + 1)
            }
        }],
        TagSpecifications=[{
            "ResourceType":
            "instance",
            "Tags": [{
                "Key": "Name",
                "Value": "CLAIRE Evidence Extractor"
            }, {
                "Key": "CLAIRE",
                "Value": "Worker",
            }, {
                "Key": "InvestigationId",
                "Value": investagtion_id,
            }]
        }],
        UserData=open(
            Path(__file__).parent /
            "./prepare_memory_capture_volume.sh").read(),
    )["Instances"][0]


def poll_extractor(investigation_id: str, instance_id: str):
    logger = get_logger(investigation_id)
    logger("Getting instance {} information".format(instance_id))
    resp = client("ec2").describe_instances(InstanceIds=[instance_id])
    logger("Instance information returned")

    return resp["Reservations"][0]["Instances"][0]


def lambda_create_extractor_handler(event: object):
    instance = create_extractor_instance(
        event["investigation_id"],
        environ["SECURITY_GROUP"],
    )
    return {
        "ready": False,
        "investigation_id": event["investigation_id"],
        "instance": instance,
    }


def lambda_extractor_ready_handler(event: object):
    event["instance"] = poll_extractor(
        event["investigation_id"],
        event["instance"]["InstanceId"],
    )
    event["ready"] = event["instance"]['State']["Name"] == "running"
    return event


def main():
    try:
        instance = create_extractor_instance(argv[1], argv[2])
        print(to_json(instance))
        while instance['State']["Name"] != "running":
            sleep(1)
            print(".", end="", flush=True)
            instance = poll_extractor(argv[1], instance["InstanceId"])

        print("running")
    except IndexError:
        print("Usage {} [investigation_id] [security_group_id]".format(
            argv[0]))


if __name__ == "__main__":
    main()