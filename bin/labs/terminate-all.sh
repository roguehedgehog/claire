#!/usr/bin/env bash

set -euo pipefail

aws ec2 terminate-instances --instance-ids \
    $(aws ec2 describe-instances \
        --filters  "Name=instance-state-name,Values=pending,running,stopped,stopping" \
        --query "Reservations[].Instances[].[InstanceId]" \
        --output text \
    | tr '\n' ' ')
    