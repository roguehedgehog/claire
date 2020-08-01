.PHONY install:
	./bin/install.sh

update:
	cd tf/claire && terraform apply -auto-approve

install-labs:
	./bin/install-labs.sh

update-labs:
	cd tf/labs/vuln-app && terraform apply -auto-approve
