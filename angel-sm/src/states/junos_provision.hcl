id = "junos_provision"

depends = [
  "common_junos_wipe",
]

state "HookJunosCLI" {
  transition {
    target = "ProvisionJunos1"
    trigger {
      type = "immediate"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "ProvisionJunos1" {
  transition {
    target = "ProvisionJunos2"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "SendLine"
      line = "configure"
    }
  }
}

state "ProvisionJunos2" {
  transition {
    target = "ProvisionJunos3"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?#"
    }
    action {
      type = "SendLine"
      line = "set system root-authentication plain-text-password"
    }
  }
}

state "ProvisionJunos3" {
  transition {
    target = "ProvisionJunos4"
    trigger {
      type   = "string"
      string = "New password:"
    }
    action {
      type = "SendLine"
      line = "Cyberwurst"
    }
  }
}

state "ProvisionJunos4" {
  transition {
    target = "ProvisionJunos5"
    trigger {
      type   = "string"
      string = "Retype new password:"
    }
    action {
      type = "SendLine"
      line = "Cyberwurst"
    }
  }
}

state "ProvisionJunos5" {
  transition {
    target = "ProvisionJunos51"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?#"
    }
    action {
      type = "SendLine"
      line = "set interfaces vme unit 0 family inet dhcp"
    }
  }
}

state "ProvisionJunos51" {
  transition {
    target = "ProvisionJunos6"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?#"
    }
    action {
      type = "SendLine"
      line = "set interfaces me0 unit 0 family inet dhcp"
    }
  }
}

state "ProvisionJunos6" {
  transition {
    target = "ProvisionJunos7"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?#"
    }
    action {
      type = "SendLine"
      line = "delete chassis auto-image-upgrade"
    }
  }
}

state "ProvisionJunos7" {
  transition {
    target = "ProvisionJunos71"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?#"
    }
    action {
      type = "SendLine"
      line = "commit"
    }
  }
}

state "ProvisionJunos71" {
  transition {
    target = "ProvisionJunos8"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?#"
    }
    action {
      type = "SendLine"
      line = "exit"
    }
  }
  # Some switches exit out of the cli during this.
  transition {
    target = "ProvisionJunos9"
    trigger {
      type  = "regex"
      regex = "root@([A-Za-z0-9\\-]+)?:(RE|LC):0%"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "ProvisionJunos8" {
  transition {
    target = "ProvisionJunos9"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "SendLine"
      line = "start shell"
    }
  }
}

state "ProvisionJunos9" {
  transition {
    target = "ProvisionJunos10"
    trigger {
      type  = "regex"
      regex = "root@([A-Za-z0-9\\-]+)?:(RE|LC):0%"
    }
    action {
      type = "Send"
      text = "set REALTTY=`tty`; set PING_TGT=\""
    }
    action {
      type = "SendConfigValue"
      key  = "provision_ping_target"
    }
    action {
      type = "SendLine"
      line = "\""
    }
  }
}

state "ProvisionJunos10" {
  transition {
    target = "ProvisionJunos11"
    trigger {
      type  = "regex"
      regex = "root@([A-Za-z0-9\\-]+)?:(RE|LC):0%"
    }
    action {
      type = "SendLine"
      line = <<EOT
/bin/sh
cat > /tmp/bootstrap.sh << EOF
#!/bin/sh
pvfail() {
  echo "PROVISION_FAILED" >> $REALTTY
  exit 1
}

while ! (ifconfig vme | grep -q inet); do echo "[BOOTSTRAP] Waiting for DHCP (1)..." ; sleep 1 ; done
while ! (ping -c 1 $PING_TGT); do echo "[BOOTSTRAP] Waiting for DHCP (2)..." ; sleep 1 ; done

echo "[BOOTSTRAP] Waiting an extra 5 seconds..."
sleep 5
echo "[BOOTSTRAP] Fetching provisioning script..."

EOT
    }
    action {
      type = "Send"
      text = "fetch -o /tmp/provision.sh \""
    }
    action {
      type = "SendConfigValue"
      key  = "provision_url"
    }
    action {
      type = "SendLine"
      line = "/provision/juniper/provision.sh\" || pvfail"
    }
    action {
      type = "SendLine"
      line = <<EOT
echo "[BOOTSTRAP] Running provisioning script..."
/bin/sh /tmp/provision.sh || pvfail
echo '[BOOTSTRAP] Provisioning complete!'
echo "PROVISION_SUCCESS" >> $REALTTY
sleep 5
echo '[BOOTSTRAP] Powering off...'
shutdown -p now
exit 0
EOF
exit
EOT
    }
  }
}

state "ProvisionJunos11" {
  transition {
    target = "ProvisionJunosRunning"
    trigger {
      type  = "regex"
      regex = "root@([A-Za-z0-9\\-]+)?:(RE|LC):0%"
    }
    action {
      type = "SendLine"
      line = "/bin/sh /tmp/bootstrap.sh"
    }
  }
}

state "ProvisionJunosRunning" {
  transition {
    target = "ProvisionJunosRunning"
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
    target = "ProvisionJunosWaitReboot"
    trigger {
      type   = "string"
      string = "PROVISION_REBOOT"
    }
  }

  transition {
    target = "ProvisionJunosFinish"
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
      type   = "string"
      string = "shutdown"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "Shutdown"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }

  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "Waiting (max 60 seconds) for system process"
    }
    action {
      type = "AddDeviceInfo"
      flag = "ProvisioningFailed"
    }
  }
}

state "ProvisionJunosFinish" {
  transition {
    target = "ProvisionJunosFinish"
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
    target = "ProvisionJunosWaitReboot"
    trigger {
      type   = "string"
      string = "PROVISION_REBOOT"
    }
  }

  transition {
    target = "JunosWaitForPoweroff"
    trigger {
      type   = "string"
      string = "shutdown"
    }
  }

  transition {
    target = "JunosWaitForPoweroff"
    trigger {
      type   = "string"
      string = "Shutdown"
    }
  }

  transition {
    target = "JunosWaitForPoweroff"
    trigger {
      type   = "string"
      string = "Waiting (max 60 seconds) for system process"
    }
  }
}

state "ProvisionJunosWaitReboot" {
  transition {
    target = "ProvisionJunosWaitReboot"
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
    target = "ProvisionJunosRebootLogin"
    trigger {
      type   = "string"
      string = "login:"
    }
    action {
      type = "SendLine"
      line = "root"
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
}

state "ProvisionJunosRebootLogin" {
  transition {
    target = "ProvisionJunosRebootLogin"
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
    target = "JunosLogin"
    trigger {
      type   = "string"
      string = "Password:"
    }
    action {
      type = "SendLine"
      line = "Cyberwurst"
    }
  }
}
