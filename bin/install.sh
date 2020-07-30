#!/usr/bin/env bash

set -euo pipefail

terraform -v
packer -v

cd tf/claire/evidence_extractor
echo "Creating Evidence Extractor Image with packer"
packer build ami.json | tee ami_details.txt
AMI_ID=$(tail ami_details.txt \
    | grep ami \
    | awk -F ' ' '{print $2}')

cd ..
if [ -f terraform.tfvars ]; then
    sed -i '/evidence_extractor_ami_id/d' terraform.tfvars
fi
echo -e "\nevidence_extractor_ami_id = \"${AMI_ID}\"" >> terraform.tfvars

echo "Installing CLAIRE in AWS with terraform"
terraform init
terraform apply
