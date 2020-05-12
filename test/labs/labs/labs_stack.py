from aws_cdk import core, aws_ec2 as ec2, aws_kms as kms, aws_rds as rds


class LabsStack(core.Stack):

    CANONICAL_USER_ID = "099720109477"

    def __init__(self, scope: core.Construct, id: str, **kwargs) -> None:
        super().__init__(scope, id, **kwargs)

        vpc = self.create_vpc()
        instance = self.create_ubuntu_instance(
            vpc,
            self.create_security_group(vpc),
        )

    def create_vpc(self) -> ec2.Vpc:
        return ec2.Vpc(
            self,
            "VPC",
            max_azs=1,
            subnet_configuration=[
                ec2.SubnetConfiguration(
                    subnet_type=ec2.SubnetType.PUBLIC,
                    name="Public",
                    cidr_mask=24,
                )
            ],
        )

    def create_security_group(self, vpc: ec2.Vpc) -> ec2.SecurityGroup:
        security_group = ec2.SecurityGroup(
            self,
            "SecurityGroup",
            vpc=vpc,
            allow_all_outbound=True,
        )
        security_group.add_ingress_rule(ec2.Peer.any_ipv4(), ec2.Port.tcp(22))

        return security_group

    def create_ubuntu_instance(self, vpc, security_group) -> ec2.Instance:
        ec2.Instance(
            self,
            "Instance",
            instance_type=ec2.InstanceType("m1.small"),
            machine_image=ec2.LookupMachineImage(
                name=
                "ubuntu-minimal/images/hvm-ssd/ubuntu-bionic-18.04-amd64-minimal-*",
                owners=[self.CANONICAL_USER_ID]),
            vpc=vpc,
            security_group=security_group,
            key_name=self.node.try_get_context("lab_key"),
        )
