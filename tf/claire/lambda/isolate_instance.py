from os import environ

from boto3 import client

from instance import InstanceService
from investigation_logger import get_logger

ec2 = client("ec2")
iam = client("iam")


def isolate(investigation_id: str, security_group):

    logger = get_logger(investigation_id)
    instance = InstanceService(investigation_id).get_instance(investigation_id)
    logger("Saving security group(s) to tag:claire_removed_groups")
    ec2.create_tags(Resources=[instance["InstanceId"]],
                    Tags=[{
                        "Key":
                        "claire_removed_groups",
                        "Value":
                        ",".join(g["GroupId"]
                                 for g in instance["SecurityGroups"])
                    }])
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

    return event
