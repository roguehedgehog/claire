#!/usr/bin/env bash

set -euo pipefail

readonly DEVICE="${1}"
readonly DEST=/mnt/mem

mkfs -t ext4 "${DEVICE}"
mount "${DEVICE}" "${DEST}"

cp /home/ubuntu/bin/avml "${DEST}"

umount "${DEST}"