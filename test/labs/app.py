#!/usr/bin/env python3

from aws_cdk import core
from os import environ

from labs.labs_stack import LabsStack


app = core.App()
LabsStack(app, "labs", env={
	'account': environ['CDK_DEFAULT_ACCOUNT'],
	'region': environ['CDK_DEFAULT_REGION']
})

app.synth()
