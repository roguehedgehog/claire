#!/usr/bin/env bash

readonly REGION=$1 # eg eu-west-2
readonly ACCOUNT=$2 # 123456789012

aws stepfunctions list-executions \
--state-machine-arn arn:aws:states:$REGION:$ACCOUNT:stateMachine:claire_investigation \
--query 'executions[].[startDate,stopDate,status]' \
--output text