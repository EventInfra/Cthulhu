#!/bin/bash
set -euo pipefail

TARGET_SWI="{{target_swi}}"

function clean_nonboot_swi() {
  echo "[PROVISION-S2-UPGRADE] Cleaning non-boot swi files..."
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
  echo "[PROVISION-S2-UPGRADE] Entered main."
  echo -en "%%%%%\"SoftwareUpdatePerformed\"%%%%%\r\n" | sudo tee $REALTTY > /dev/null
  clean_nonboot_swi
  export SKU="$(awk -F" " '/^Sku: / {print $2}' /etc/fdl)"
  if [[ "x$SKU" == "xDCS-7150S-"* ]]; then
    echo "[PROVISION-S2-UPGRADE] Special case: Delete all SWI before update... THIS MAKES THE SWITCH UNBOOTABLE"
    rm -v /mnt/flash/*.swi
  fi
  echo "[PROVISION-S2-UPGRADE] Fetching new OS..."
  curl -o "/mnt/flash/$TARGET_SWI" "{{base_url}}/provision/arista/swi/$TARGET_SWI"
  echo "[PROVISION-S2-UPGRADE] Writing out boot config..."
  cat > /mnt/flash/boot-config <<EOF
CONSOLESPEED=9600
SWI=flash:/$TARGET_SWI
EOF
  echo "[PROVISION-S2-UPGRADE] Finished OS upgrade!"
  echo -en "PROVISION_REBOOT\r\n" | sudo tee $REALTTY > /dev/null
}

main "$@"
