id = "arista_provision"

depends = [
  "arista_wipe"
]

state "AristaBootloaderHook" {
  transition {
    target = "AristaBootloaderHookExit"
    trigger {
      type = "immediate"
    }
    action {
      type = "SendLine"
      line = "echo DISABLE=True > /mnt/flash/zerotouch-config"
    }
  }
}

state "AristaBootloaderHookExit" {
  transition {
    target = "AristaBootloaderExit"
    trigger {
      type = "string"
      string = "Aboot#"
    }
  }
}

state "HookAristaCLI" {
  transition {
    target = "ProvisionAristaEnterBash"
    trigger {
      type = "immediate"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "ProvisionAristaEnterBash" {
  transition {
    target = "ProvisionAristaBash0"
    trigger {
      type   = "string"
      string = "localhost#"
    }
    action {
      type = "SendLine"
      line = "bash"
    }
  }
}

state "ProvisionAristaBash0" {
  transition {
    target = "ProvisionAristaBash1"
    trigger {
      type   = "string"
      string = "[admin@localhost ~]$"
    }
    action {
      type = "SendLine"
      line = "export REALTTY=\"$(tty)\""
    }
  }
}

state "ProvisionAristaBash1" {
  transition {
    target = "ProvisionAristaBash2"
    trigger {
      type   = "string"
      string = "[admin@localhost ~]$"
    }
    action {
      type = "SendLine"
      line = <<EOT
cat > /mnt/flash/bootstrap.sh << EOF
#!/bin/bash
function pvfail() {
  echo -en "PROVISION_FAILED\r\n" | sudo tee $REALTTY
  exit 1
}

# Sleep to avoid interleaving output
sleep 5

echo "[BOOTSTRAP] Performing DHCP..."
sudo dhclient -v ma1 || pvfail
echo "[BOOTSTRAP] Fetching provisioning script..."

EOT
    }
    action {
      type = "Send"
      text = "curl -o /mnt/flash/provision.sh \""
    }
    action {
      type = "SendConfigValue"
      key  = "provision_url"
    }
    action {
      type = "SendLine"
      line = "/provision/arista/provision.sh\" || pvfail"
    }
    action {
      type = "SendLine"
      line = <<EOT

echo "[BOOTSTRAP] Running provisioning script..."
/mnt/flash/provision.sh || pvfail
echo '[BOOTSTRAP] Provisioning complete!'
echo -en "PROVISION_SUCCESS\r\n" | sudo tee $REALTTY
echo '[BOOTSTRAP] Waiting a bit...'
sleep 30
echo '[BOOTSTRAP] Rebooting...'
sudo reboot
exit 0
EOF
EOT
    }
  }
}

state "ProvisionAristaBash2" {
  transition {
    target = "ProvisionAristaBash3"
    trigger {
      type   = "string"
      string = "[admin@localhost ~]$"
    }
    action {
      type = "SendLine"
      line = "nohup /mnt/flash/bootstrap.sh 0<&- &>/mnt/flash/provision.log &"
    }
  }
}

state "ProvisionAristaBash3" {
  transition {
    target = "ProvisionAristaRunning"
    trigger {
      type   = "string"
      string = "[admin@localhost ~]$"
    }
    action {
      type = "SendLine"
      line = "sleep 3 ; tail -fn1000 /mnt/flash/provision.log"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "PROVISION_FAILED"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }

  transition {
    target = "AristaWaitForReboot"
    trigger {
      type   = "string"
      string = "PROVISION_REBOOT"
    }
  }

  transition {
    target = "ProvisionAristaBash3"
    trigger {
      type   = "string"
      string = "PROVISION_SUCCESS"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningSuccess"
    }
  }
}

state "ProvisionAristaRunning" {
  transition {
    target = "ProvisionAristaRunning"
    trigger {
      type = "regex"
      regex = "%%%%%(?<devinfo>[^%]+)%%%%%"
    }
    action {
      type = "Function"
      func = "ArbitraryDeviceInfo"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "PROVISION_FAILED"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }

  transition {
    target = "AristaWaitForReboot"
    trigger {
      type   = "string"
      string = "PROVISION_REBOOT"
    }
  }

  transition {
    target = "ProvisionAristaFinish"
    trigger {
      type   = "string"
      string = "PROVISION_SUCCESS"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningSuccess"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type = "string"
      string = "login:"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }
}

state "ProvisionAristaFinish" {
  transition {
    target = "ProvisionAristaFinish"
    trigger {
      type = "regex"
      regex = "%%%%%(?<devinfo>[^%]+)%%%%%"
    }
    action {
      type = "Function"
      func = "ArbitraryDeviceInfo"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "PROVISION_FAILED"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }

  transition {
    target = "AristaWaitForReboot"
    trigger {
      type   = "string"
      string = "PROVISION_REBOOT"
    }
  }

  transition {
    target = "ProvisionAristaFinish"
    trigger {
      type   = "string"
      string = "PROVISION_SUCCESS"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningSuccess"
    }
  }

  transition {
    target = "ProvisionAristaRestart"
    trigger {
      type = "string"
      string = "Aboot"
    }
  }
}

state "ProvisionAristaRestart" {
  transition {
    target = "ProvisionAristaRestart"
    trigger {
      type = "regex"
      regex = "%%%%%(?<devinfo>[^%]+)%%%%%"
    }
    action {
      type = "Function"
      func = "ArbitraryDeviceInfo"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type = "string"
      string = "login:"
    }
  }
}
