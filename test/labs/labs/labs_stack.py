from aws_cdk import (
    core,
    aws_ec2 as ec2,
    aws_kms as kms,
    aws_rds as rds
)


class LabsStack(core.Stack):

    CANONICAL_USER_ID = "099720109477"

    def __init__(self, scope: core.Construct, id: str, **kwargs) -> None:
        super().__init__(scope, id, **kwargs)

        vpc = ec2.Vpc(self, "VPC", max_azs=1)
        key = kms.Key(self, "Key", removal_policy=core.RemovalPolicy.DESTROY)
        sec_group = ec2.SecurityGroup(self, "SecurityGroup",
                                      vpc=vpc,
                                      allow_all_outbound=True
                                      )
        sec_group.add_ingress_rule(ec2.Peer.any_ipv4(), ec2.Port.tcp(22))
        instance = ec2.Instance(self, "Instance",
                                instance_type=ec2.InstanceType("m1.small"),
                                machine_image=ec2.LookupMachineImage(
                                    name="ubuntu-minimal/images/hvm-ssd/ubuntu-bionic-18.04-amd64-minimal-*",
                                    owners=[self.CANONICAL_USER_ID]
                                ),
                                vpc=vpc,
                                security_group=sec_group,
                                key_name="Key"  
                                )
