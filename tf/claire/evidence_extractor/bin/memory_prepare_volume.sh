#!/usr/bin/env bash

set -euo pipefail

readonly DEVICE="${1}"

mkfs -t ext4 "${DEVICE}"
mount "${DEVICE}" /mnt/mem

mkdir /mnt/mem/memory
cp /home/ubuntu/bin/avml /mnt/mem/memory/

umount /mnt/mem