#!/usr/bin/env bash

aws ec2 describe-instances \
    --query 'Reservations[*].Instances[*].[InstanceId]' \
    --filters Name=instance-state-name,Values=running \
    --output text \
    | while read line; 
        do claire investigate $line "Bulk Test" & sleep 1;  
    done