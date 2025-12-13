#!/bin/sh

pfail() {
        echo "[PROVISION-S1] Failure!"
        exit 1
}

echo "[PROVISION-S1] Entered main."
export OS_VERSION="$(sysctl kern.osrelease | awk '{ print $2 }')"
export SKU="$(sysctl hw.product.model | awk '{ print $2 }')"
if [ "x$SKU" == "xpvi-model" ]; then
  echo "[PROVISION-S1] Using alternate OS detection..."
  export SKU="$(sysctl hw.product.pvi_model | awk '{ print $2 }')"
  export OS_VERSION="$(echo "show version" | cli | grep "Junos:" | awk '{ print $2 }')"
fi
echo "[PROVISION-S1] Detected JunOS version: $OS_VERSION"
echo "[PROVISION-S1] Detected SKU: $SKU"
echo "[PROVISION-S1] Downloading second stage..."
fetch -o /tmp/provision-stage2.sh "{{base_url}}/provision/juniper/stage2.sh?sku=$SKU&junos=$OS_VERSION" || pfail
echo "[PROVISION-S1] Chaining into second stage..."
sh /tmp/provision-stage2.sh
