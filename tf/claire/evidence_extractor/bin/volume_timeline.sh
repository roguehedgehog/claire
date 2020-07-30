#!/usr/bin/env bash

set -euo pipefail

readonly DIR=/home/ubuntu/investigation
readonly DEVICE=$1
readonly NAME=$2

tsk_gettimes "${DEVICE}" \
    | mactime -d -y \
        -p "${DIR}/artifacts/passwd" \
        -g "${DIR}/artifacts/group" \
    | gzip > "${DIR}/timeline/${NAME}.csv.gz"