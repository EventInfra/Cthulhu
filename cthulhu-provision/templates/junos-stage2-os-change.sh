#!/bin/sh

pfail() {
        echo "[PROVISION-S2-UPGRADE] Failure!"
        exit 1
}

echo "[PROVISION-S2-UPGRADE] Performing storage cleanup..."
echo -e "request system storage cleanup\nyes" | cli
echo 'request system snapshot delete snap*' | cli

echo "[PROVISION-S2-UPGRADE] Upgrading OS image..."
echo "request system software add no-validate no-copy reboot {{base_url}}/provision/juniper/jinstall/{{target_jinstall}}" | cli > /tmp/update.log

echo "[PROVISION-S2-UPGRADE] Upgrade log:"
cat /tmp/update.log

if grep -iq error /tmp/update.log; then
        echo "[PROVISION-S2-UPGRADE] Failure in update log!"
        pfail
fi
echo "[PROVISION-S2-UPGRADE] Rebooting..."
echo "PROVISION_REBOOT"
echo -e "request system power-off at now\nyes" | cli
sleep 3600
