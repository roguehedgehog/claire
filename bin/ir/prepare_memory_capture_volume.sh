#!/usr/bin/env bash

set -eu

mkdir -p /mnt/mem
mkfs -t ext4 /dev/xvdm
mount /dev/xvdbm /mnt/mem
mkdir /mnt/mem/memory
cd /mnt/mem/memory
wget https://github.com/microsoft/avml/releases/download/v0.2.0/avml
chmod +x avml
cd /
umount /mnt/mem