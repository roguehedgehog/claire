#!/usr/bin/env bash

set -euo pipefail

terraform -v
packer -v

readonly VULN_DRUPAL_AMI=$(aws ec2 describe-images \
        --filters Name=name,Values="Vulnerable Drupal Instance" \
        | jq -r .Images[0].ImageId)

if [ "$CLAIRE_AMI" != null ]; then
    echo "Removing lab AMI ${VULN_DRUPAL_AMI}"
    aws ec2 deregister-image --image-id "$VULN_DRUPAL_AMI"
fi

cd tf/labs/vuln-app/ami
echo "Creating Vulnerable Drupal Image with packer"
#packer build ami.json | tee ami_details.txt
AMI_ID=$(tail ami_details.txt \
    | grep ami \
    | awk -F ' ' '{print $2}')

cd ../
if [ -f terraform.tfvars ]; then
    sed -i '/vulnerable_ami_id/d' terraform.tfvars
fi

echo -e "\nvulnerable_ami_id = \"${AMI_ID}\"" >> terraform.tfvars

echo "Starting vuln app with terraform"
terraform init
terraform apply
