#!/usr/bin/env bash

set -euo pipefail

main () {
    # https://www.packer.io/docs/other/debugging.html#issues-installing-ubuntu-packages
    while [ ! -f /var/lib/cloud/instance/boot-finished ]; do echo 'Waiting for cloud-init...'; sleep 5; done

    create_directories
    install_tools
}

create_directories () {
    sudo mkdir -p \
        /mnt/{mem,vol} \
        /home/ubuntu/investigation/{timeline,logs,artifacts,memory}
}

install_tools () {
    echo "Install requirements"
    sudo apt update
    sudo apt install --yes --no-install-recommends \
        build-essential \
        awscli \
        sleuthkit \
        unzip \
        python2.7 \
        python-dev

    echo "Download AVML"
    wget https://github.com/microsoft/avml/releases/download/v0.2.0/avml \
        --directory-prefix=/home/ubuntu/bin

    wget https://github.com/microsoft/avml/releases/download/v0.2.0/avml-convert \
        --directory-prefix=/home/ubuntu/bin

    echo "Make programs executable add to path"
    chmod -R +x /home/ubuntu/bin/
    sudo cp /home/ubuntu/bin/* /usr/local/bin/

    install_volatility
}

install_volatility () {
    echo "Install Volatility"
    mkdir -p /opt
    echo "Download"
    wget https://github.com/volatilityfoundation/volatility/archive/master.zip \
        --directory-prefix=/tmp

    echo "Unzip"
    unzip /tmp/master.zip -d /tmp

    sudo mv /tmp/volatility-master /opt/vol
    sudo chown -R ubuntu:ubuntu /opt/vol

    chmod +x /opt/vol/vol.py

    echo "Copy profiles"
    cp /home/ubuntu/profiles/* /opt/vol/volatility/plugins/overlays/linux/


    echo "Speed up by removing windows junk"
    cd /opt/vol/volatility/plugins/overlays/windows
    rm *x64* *x86*
    cd -

    sudo ln -s $(which python2.7) /usr/local/bin/python
    sudo ln -s /opt/vol/vol.py /usr/local/bin/vol

    echo "Install Optional Requirements"
    wget https://bootstrap.pypa.io/get-pip.py --directory-prefix=/tmp
    sudo python /tmp/get-pip.py
    pip install pycryptodome
    pip install distorm3==3.4.4
}


main