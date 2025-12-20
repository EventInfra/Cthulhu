#!/bin/sh

pfail() {
        echo "[PROVISION-S2-UPGRADE] Failure!"
        exit 1
}

echo "[PROVISION-S2-UPGRADE] Configuring NTP..."
{
  echo "configure"
  echo "set system ntp server {{ntp_server}}"
  echo "set system ntp threshold 600 action accept"
  echo "commit and-quit"
  } | /usr/sbin/cli

echo "%%%%%\"SoftwareUpdatePerformed\"%%%%%"
echo "PROVISION_REBOOT"
echo "[PROVISION-S2-UPGRADE] Performing storage cleanup..."
echo -e "request system storage cleanup\nyes" | cli
echo 'request system snapshot delete snap*' | cli

echo "[PROVISION-S2-UPGRADE] Upgrading OS image..."
if sysctl hw.product.pvi_model 2>/dev/null | grep -q "hw.product.pvi_model: ex2300-c" ; then
  fetch -o /.mount/tmp/ex2300c-image.tgz "{{base_url}}/provision/juniper/jinstall/{{target_jinstall}}"
  echo "request system software add /.mount/tmp/ex2300c-image.tgz no-copy no-validate force unlink" | cli > /tmp/update.log
else
  echo "request system software add no-validate no-copy {{base_url}}/provision/juniper/jinstall/{{target_jinstall}}" | cli > /tmp/update.log
fi

echo "[PROVISION-S2-UPGRADE] Upgrade log:"
cat /tmp/update.log

if grep -vi "Error: Secure Boot status is unknown" /tmp/update.log | grep -iq error; then
        echo "[PROVISION-S2-UPGRADE] Failure in update log!"
        pfail
fi

echo "[PROVISION-S2-UPGRADE] Rebooting..."
echo -e "request system reboot at now\nyes" | cli
sleep 3600

}