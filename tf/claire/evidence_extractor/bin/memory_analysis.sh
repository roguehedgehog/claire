#!/usr/bin/env bash

set -euo pipefail

readonly DEVICE=$1
readonly DEST=$2
readonly INVESTIGATION=/home/ubuntu/investigation/memory
readonly IMG=/tmp/memory.lime


declare -a plugins=(
    "bash"
    "bash_env"
    "dmesg"
    "getcwd"
    "ifconfig"
    "iomem"
    "kernel_opened_files"
    "lsmod"
    "lsof"
    "netstat"
    "proc_maps"
    "psaux"
    "psenv"
    "psscan"
    "pslist"
)

main () {
    echo "Copying image from device"
    save_image

    echo "Uncompressing image"
    prepare_image 

    echo "Running volatility analysis"
    run_analysis

    echo "Uploading results to s3"
    aws s3 sync "${INVESTIGATION}" "${DEST}/memory"
}

save_image() {
    mount -o ro "${DEVICE}" /mnt/mem
    cp /mnt/mem/memory.lime.compressed "${INVESTIGATION}"
    umount /mnt/mem
}

prepare_image() {
    avml-convert "${INVESTIGATION}/memory.lime.compressed" "${IMG}"
}

run_analysis() {
    echo "Searching for volatility profile"
    vol --info \
        | grep "Profile.*Linux" \
        | awk -F ' ' '{print $1}' \
        | while read -r profile; do
            if vol --profile="$profile" -f ${IMG} linux_banner; then
                echo "Found ${profile}"
                run_plugins "${profile}"
                break
            fi
        done
}

run_plugins() {
    readonly profile="${1}"
    for plugin in "${plugins[@]}"
    do
        echo "$(date) Running ${plugin}"
        vol --profile "${profile}" -f "${IMG}" "linux_${plugin}" > "${INVESTIGATION}/${plugin}"
    done
    echo "$(date) Plugin run complete"
}

main
