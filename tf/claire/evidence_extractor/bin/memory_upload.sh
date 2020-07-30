#!/usr/bin/env bash

set -euo pipefail

readonly DEVICE="${1}"
readonly DEST="${2}"

mount -o ro "${DEVICE}" /mnt/mem
aws s3 cp /mnt/mem/memory/memory.lime "${DEST}"
umount /mnt/mem
