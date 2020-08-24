#!/usr/bin/env bash

set -euo pipefail

readonly DIR=/home/ubuntu/investigation
readonly VOL=$1
readonly NAME=$2
readonly DEVICE="$(lsblk -o +SERIAL | grep ${VOL//-/} | awk -F ' ' '{print $1}')"

tsk_gettimes "/dev/${DEVICE}" \
    | mactime -d -y \
        -p "${DIR}/artifacts/passwd" \
        -g "${DIR}/artifacts/group" \
    | gzip > "${DIR}/timeline/${NAME}.csv.gz"