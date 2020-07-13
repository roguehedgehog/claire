#!/usr/bin/env python3

from boto3 import client
from os import environ
from logging import getLogger, INFO
from datetime import datetime
from sys import argv
from json import dumps

logger = getLogger()
info = lambda msg, investigation_id=None: logger.info(
    msg,
    extra={
        "referrer": "claire",
        "investigation_id": investigation_id
    },
)


def lambda_handler(event, context):
    info("Received event {}".format(event["id"]))
    if not is_investigatable(event):
        info("Event {} {} will not be investigated".format(
            event["source"],
            event["id"],
        ))
        return

    info("Event {} {} can be investigated.".format(
        event["source"],
        event["id"],
    ))

    instance_id = event["detail"]["resource"]["instanceDetails"]["instanceId"]
    investigation_id = create_investigation(instance_id, event)

    return {"investigation_id": investigation_id, "instance_id": instance_id}


def is_investigatable(event: object) -> bool:
    return "resourceType" in event["detail"]["resource"] and \
        event["detail"]["resource"]["resourceType"] == "Instance"


def create_investigation(instance_id: str, details) -> str:
    ec2 = client("ec2")
    s3 = client("s3")

    instance = get_instance(ec2, instance_id)
    investigation_id = "{}_{}".format(datetime.now(), instance["InstanceId"])

    put(s3, investigation_id, "alert.json", details)
    put(s3, investigation_id, "instance.json", instance)

    tag(
        ec2,
        investigation_id,
        instance_id,
        [{
            "Key": "CLARE",
            "Value": "Investigating"
        }, {
            "Key": "InvestigationId",
            "Value": investigation_id
        }],
    )

    return investigation_id


def get_instance(ec2: object, instance_id: str) -> object:
    info("Getting instance details for {}".format(instance_id))
    resp = ec2.describe_instances(InstanceIds=[instance_id])
    instance = resp["Reservations"][0]["Instances"][0]
    info("Instance {} found".format(instance_id))

    return instance


def put(s3: object, investigation_id: str, name: str, details: object):
    info("Putting {}".format(name), investigation_id)
    s3.put_object(
        Bucket=environ["INVESTIGATION_BUCKET"],
        Key="{}/{}".format(investigation_id, name),
        Body=dumps(details, indent=2, skipkeys=True, default=str),
    )
    info("Putting {} complete".format(name), investigation_id)


def tag(ec2: object, investigation_id: str, instance_id: str, tags: list):
    info("Tagging suspicious instance", investigation_id)
    ec2.create_tags(Resources=[instance_id], Tags=tags)
    info("Tagging complete", investigation_id)


if __name__ == "__main__":
    if len(argv) == 1:
        print(
            "You must provide an instance id to create an investigation, {} [instance id]"
            .format(argv[0]))
        exit(1)

    if "INVESTIGATION_BUCKET" not in environ:
        print(
            "INVESTIGATION_BUCKET must be set, export INVESTIGATION_BUCKET=[bucket]"
        )
        exit(1)

    create_investigation(argv[1], {"details": "Manually triggered"})