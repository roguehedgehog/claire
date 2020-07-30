#!/usr/bin/env bash

set -euo pipefail

readonly DIR=/home/ubuntu/investigation
readonly DEVICE=$1
readonly NAME=$2

main () {
    mount -o ro "${DEVICE}" /mnt/vol

    extract_logs
    extract_artifacts
    volume_timeline.sh "${DEVICE}" "root-${NAME}"

    umount /mnt/vol
}

function extract_logs {
    cd /mnt/vol/var/log
    find . -type f -exec grep -Iq . {} + -print \
        | cpio -pdm ${DIR}/logs/

    gzip -r ${DIR}/logs/

    last -f /mnt/vol/var/log/wtmp > ${DIR}/logs/wtmp # system power logs
    last -f /mnt/vol/var/log/btmp > ${DIR}/logs/btmp # login attempts
    cd -
}

function extract_artifacts {
    cd /mnt/vol/etc/
    cp {passwd,group,shadow,hostname,timezone,hosts,resolv.conf,*-release,sudoers,fstab,crontab} \
        ${DIR}/artifacts/ || true
    cd -
}

main
