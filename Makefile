.PHONY: install update install-labs update-labs

install:
	./bin/install.sh

update:
	cd tf/claire && terraform apply -auto-approve

install-labs:
	./bin/install-labs.sh

update-labs:
	cd tf/labs && terraform apply -auto-approve
