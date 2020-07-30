#!/usr/bin/env bash

set -euo pipefail

main () {
    # https://www.packer.io/docs/other/debugging.html#issues-installing-ubuntu-packages
    while [ ! -f /var/lib/cloud/instance/boot-finished ]; do echo 'Waiting for cloud-init...'; sleep 1; done

    create_directories
    install_tools
}

create_directories () {
    sudo mkdir -p \
        /mnt/{mem,vol} \
        /home/ubuntu/investigation/{timeline,logs,artifacts}
}

install_tools () {
    sudo apt update
    sudo apt install --yes --no-install-recommends \
        awscli \
        sleuthkit \

    wget https://github.com/microsoft/avml/releases/download/v0.2.0/avml \
        --directory-prefix=/home/ubuntu/bin

    chmod -R +x /home/ubuntu/bin/
    sudo cp /home/ubuntu/bin/* /usr/local/bin/
}

main