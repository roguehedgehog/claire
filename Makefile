SHELL=/usr/bin/env bash

.PHONY install:
	remove-claire-ami

	terraform -v
	packer -v

	cd tf/claire/evidence_extractor
	@echo "Creating Evidence Extractor Image with packer"
	packer build ami.json | tee ami_details.txt
	AMI_ID := $(tail ami_details.txt \
		| grep ami \
		| awk -F ' ' '{print $2}')

	cd ..
	if [ -f terraform.tfvars ]; then
		sed -i '/evidence_extractor_ami_id/d' terraform.tfvars
	fi
	echo -e "\nevidence_extractor_ami_id = \"${AMI_ID}\"" >> terraform.tfvars

	@echo "Installing CLAIRE in AWS with terraform"
	terraform init
	terraform apply

update:
	cd tf/claire && terraform apply -auto-approve

install-labs:
	./bin/install-labs.sh

update-labs:
	cd tf/labs/vuln-app && terraform apply -auto-approve

remove-claire-ami:
	readonly CLAIRE_AMI=$(aws ec2 describe-images \
			--filters Name=name,Values="CLAIRE Evidence Extractor" \
			| jq -r .Images[0].ImageId)

	if [ "$CLAIRE_AMI" != null ]; then
		echo "Removing existing extractor AMI ${CLAIRE_AMI}"
		aws ec2 deregister-image --image-id "$CLAIRE_AMI"
	fi
