#!/usr/bin/env bash

set -euo pipefail

terraform -v
packer -v

readonly LAB_AMI=$(aws ec2 describe-images \
        --filters Name=name,Values="CLAIRE Vulnerable Lab Server" \
        | jq -r .Images[0].ImageId)

if [ "$LAB_AMI" != null ]; then
    echo "Removing lab AMI ${LAB_AMI}"
    aws ec2 deregister-image --image-id "$LAB_AMI"
fi

cd tf/labs/ami
echo "Creating Vulnerable Server Image with packer"
#packer build ami.json | tee ami_details.txt
AMI_ID=$(tail ami_details.txt \
    | grep ami \
    | awk -F ' ' '{print $2}')

cd ../
if [ -f terraform.tfvars ]; then
    sed -i '/lab_ami_id/d' terraform.tfvars
fi

echo -e "\nlab_ami_id = \"${AMI_ID}\"" >> terraform.tfvars

if [ ! -f dist/lab_key ]; then
    echo "Generating keypair for labs in tf/labs/ami/dist/lab_key"
    ssh-keygen -q -N '' -t rsa -b 4096 -f dist/lab_key
fi

echo "Starting lab with terraform"
terraform init
terraform apply
