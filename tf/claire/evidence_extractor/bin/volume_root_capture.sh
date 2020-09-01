#!/usr/bin/env bash

set -euo pipefail

readonly DIR=/home/ubuntu/investigation
readonly VOL=$1
readonly NAME=$2

main () {
    local DEVICE="$(lsblk -o +SERIAL | grep ${VOL//-/} | awk -F ' ' '{print $1}')"
    mount -o ro "/dev/${DEVICE}p1" /mnt/vol

    extract_logs
    extract_artifacts
    volume_timeline.sh "${VOL}" "root-${NAME}"

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
    cd "${DIR}/artifacts"
    find /mnt/vol -type f \
        -name "known_hosts" \
        -o -name "authorized_keys" \
        -o -name ".bashrc" \
        -o -name ".profile" \
        -o -name ".bash_history" \
        -o -name ".bashrc" \
        -o -path "/etc/passwd" \
        -o -path "/etc/group" \
        -o -path "/etc/shadow" \
        -o -path "/etc/hostname" \
        -o -path "/etc/timezone" \
        -o -path "/etc/hosts" \
        -o -path "/etc/resolv.conf" \
        -o -path "/etc/sudoers" \
        -o -path "/etc/fstab" \
        -o -path "/etc/crontab" \
        -o -ipath "/etc/*-release" \
        -o -ipath "/etc/cron.d/*" \
        -o -ipath "/var/spool/cron/crontabs/*" \
        2>/dev/null \
        | sed -e "p; s:/:_:g" \
        | xargs -n2 cp
    cd -
}

main
