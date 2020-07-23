#!/usr/bin/env python3

from boto3 import client
from investigation_logger import get_logger, to_json, log_to_console


def get_command_status(investigation_id: str, instance_id: str,
                       command_id: str):
    ssm = client("ssm")
    log = get_logger(investigation_id)
    log("Checking command {} status".format(command_id))
    resp = ssm.get_command_invocation(
        InstanceId=instance_id,
        CommandId=command_id,
    )
    log("Command {} status returned {}".format(command_id, resp))

    return resp["Status"]


def lambda_is_command_complete(event: object):
    event["command_status"] = get_command_status(
        event["investigation_id"],
        event["running_command_instance"],
        event["running_command_id"],
    )
    event["is_ready"] = event["command_status"] not in [
        "Pending", "Delayed", "InProgress"
    ]

    if event["is_ready"] is True:
        if event["command_status"] != "Success":
            raise RuntimeError("Command {}".format(event["command_status"]))

    return event