#!/bin/bash
set -euo pipefail

function apply_config() {
        echo "[PROVISION-S2] Applying configuration..."
        (FastCli -p 15 -c  "configure replace {{base_url}}/configuration/arista/${SERIAL_NUMBER}" | tee /mnt/flash/provision-configure.log) || return 1
        echo "[PROVISION-S2] Application complete!..."
        if grep -iq "error" /mnt/flash/provision-configure.log ; then
                echo "[PROVISION-S2] Detected error in configuration. Exiting..."
                return 1
        fi
        if grep -iq "invalid" /mnt/flash/provision-configure.log ; then
                echo "[PROVISION-S2] Detected error in configuration. Exiting..."
                return 1
        fi
        echo "[PROVISION-S2] Saving..."
        FastCli -p 15 -c "write mem"
}

function clean_nonboot_swi() {
  echo "[PROVISION-S2] Cleaning non-boot swi files..."
  pushd /mnt/flash
  CUR_FILE=$(cat boot-config | grep SWI | cut -d/ -f2)
  for i in *.swi ; do
    if [[ "x$i" != "x$CUR_FILE" ]]; then
      rm -v "$i"
    fi
  done
  popd
}

function main() {
        echo "[PROVISION-S2] Entered main."
        clean_nonboot_swi
        SERIAL_NUMBER=$(awk -F" " '/^SerialNumber: / {print $2}' /etc/fdl)
        echo "[PROVISION-S2] Detected serial number: $SERIAL_NUMBER"
        for file in autoreload.py autoreload.sh autoreload_legacy.sh; do
                echo "[PROVISION-S2] Fetching file $file to /mnt/flash/$file..."
                curl -o "/mnt/flash/$file" "{{base_url}}/provision/arista/assets/$file"
        done
        S=0
        for i in 1 2 3 4 5; do
                S=1
                apply_config && break || true
                S=0
                echo "[PROVISION-S2] Retrying failed..."
                sleep 15
        done
        if [[ "$S" == "0" ]]; then
                echo "[PROVISION-S2] Failed to apply config!"
                exit 1
        fi
        echo "[PROVISION-S2] Finished!"
}

main "$@"
