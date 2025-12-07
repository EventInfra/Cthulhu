#!/bin/bash
set -euo pipefail
function main() {
        echo "[PROVISION-S1] Entered main."
        export EOS_VERSION="$(cat /etc/swi-version | grep SWI_VERSION | cut -d= -f2)"
        echo "[PROVISION-S1] Detected EOS version: $EOS_VERSION"
        export SKU="$(awk -F" " '/^Sku: / {print $2}' /etc/fdl)"
        echo "[PROVISION-S1] Detected SKU: $SKU"
        echo "[PROVISION-S1] Downloading second stage..."
        curl -o /mnt/flash/provision-stage2.sh --get -d "eos=$EOS_VERSION" -d "sku=$SKU" "{{base_url}}/provision/arista/stage2.sh"
        echo "[PROVISION-S1] Chaining into second stage..."
        bash /mnt/flash/provision-stage2.sh
}

main "$@"
