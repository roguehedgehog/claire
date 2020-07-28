#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console
from get_instance import InstanceService
from manage_volumes import MoveVolumesRequst
from terminate_instance import terminate_instance
from sys import argv
from os import environ
from pathlib import Path
from time import sleep

UBUNTU_2004_64_HVM = "ami-0127d62154efde733"


def create_extractor_instance(investagtion_id: str):
    ec2 = client("ec2")
    instance_service = InstanceService(investagtion_id)
    instance = instance_service.get_instance(investagtion_id)
    volumes = instance_service.get_volumes(instance)
    memory_size = instance_service.get_memory_size(instance)
    instance_service.logger("Creating extractor instance")
    extractor = ec2.run_instances(
        ImageId=UBUNTU_2004_64_HVM,
        InstanceType="t2.small",
        MinCount=1,
        MaxCount=1,
        InstanceInitiatedShutdownBehavior="terminate",
        SubnetId=instance["NetworkInterfaces"][0]["SubnetId"],
        KeyName="vuln-app-key",
        BlockDeviceMappings=[{
            "DeviceName": "/dev/sda1",
            "Ebs": {
                "DeleteOnTermination": True,
                "VolumeSize": 8 + max(volumes, key=lambda v: v["Size"])["Size"]
            }
        }, {
            "DeviceName": "/dev/sdm",
            "Ebs": {
                "DeleteOnTermination": True,
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
        IamInstanceProfile={
            "Arn": environ["IAM_PROFILE"],
        })["Instances"][0]

    instance_service.logger("extractor instance created")

    return extractor


def poll_extractor(investigation_id: str, instance_id: str):
    logger = get_logger(investigation_id)
    logger("Getting instance {} information".format(instance_id))
    resp = client("ec2").describe_instances(InstanceIds=[instance_id])
    logger("Instance information returned")

    return resp["Reservations"][0]["Instances"][0]


def lambda_handler(event: object, context):
    instance = create_extractor_instance(event["investigation_id"])
    return {
        **event,
        **{
            "is_ready": False,
            "extractor_id": instance["InstanceId"],
        }
    }


def lambda_is_extractor_ready(event: object, context):
    instance = poll_extractor(event["investigation_id"], event["extractor_id"])

    event["is_ready"] = instance['State']["Name"] == "running"
    if event["is_ready"]:
        event["memory_volume_id"] = [
            device["Ebs"]["VolumeId"]
            for device in instance["BlockDeviceMappings"]
            if device["DeviceName"] == "/dev/sdm"
        ][0]
        event["move_volumes"] = MoveVolumesRequst(
            event["investigation_id"],
            [{
                "volume_id": event["memory_volume_id"],
                "device": "/dev/xvdm"
            }],
            event["extractor_id"],
            event["instance_id"],
        ).asdict()

    return event


def lambda_terminate_extractor(event: object, context):
    terminate_instance(event[0]["investigation_id"], event[0]["extractor_id"])

    return event


def main():

    log_to_console()
    try:
        event = {
            "investigation_id":
            argv[1],
            "instance_id":
            InstanceService(argv[1]).get_instance(argv[1])["InstanceId"]
        }
        environ["IAM_PROFILE"] = argv[2]
        environ["SECURITY_GROUP_ID"] = argv[3]
    except IndexError:
        print(
            "Usage {} [investigation_id] [iam_profile_arn] [security_group_id]"
            .format(argv[0]))
        return 1

    event = lambda_handler(event, {})
    while event["is_ready"] is False:
        sleep(5)
        event = lambda_is_extractor_ready(event, {})

    print(to_json(event))


if __name__ == "__main__":
    main()