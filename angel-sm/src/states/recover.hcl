id = "recover"


state "SwitchDetect" {
  merge = "append"
  transition {
    target = "LegacyJunosUBoot1"
    trigger {
      type   = "string"
      string = "U-Boot 1.1"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "LegacyJunosUBoot1"
    trigger {
      type   = "string"
      string = "U-Boot 2010.03"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
}

state "LegacyJunosUBoot1" {
  transition {
    target = "LegacyJunosUBoot2"
    trigger {
      type   = "string"
      string = "scanning bus for devices"
    }
    action {
      type = "SendControl"
      char = "c"
    }
  }
}

state "LegacyJunosUBoot2" {
  transition {
    target = "LegacyJunosUBoot3"
    trigger {
      type   = "string"
      string = "=>"
    }
    action {
      type = "SendLine"
      line = "printenv"
    }
  }
}

state "LegacyJunosUBoot3" {
  transition {
    target = "LegacyJunosUBoot4"
    trigger {
      type   = "string"
      string = "=>"
    }
    action {
      type = "SendLine"
      line = "setenv boot_unattended"
    }
  }
}

state "LegacyJunosUBoot4" {
  transition {
    target = "LegacyJunosLoader1"
    trigger {
      type   = "string"
      string = "=>"
    }
    action {
      type = "SendLine"
      line = "boot"
    }
  }
}

state "LegacyJunosLoader1" {
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "can't load '/kernel'"
    }
    action {
      type = "AddDeviceInfo"
      flag = "OSCorruption"
    }
  }
  transition {
    target = "LegacyJunosLoader2"
    trigger {
      type   = "string"
      string = "space bar for command prompt"
    }
    action {
      type  = "Repeat"
      times = 10

      action {
        type = "Send"
        text = " "
      }

      action {
        type = "Flush"
      }
    }
  }
}

state "LegacyJunosLoader2" {
  transition {
    target = "LegacyJunosLoader3"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "LegacyJunosLoader3" {
  transition {
    target = "LegacyJunosLoader4"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "Send"
      text = "set ipaddr="
    }
    action {
      type = "SendConfigValue"
      key  = "tftp_device_ip"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "LegacyJunosLoader4" {
  transition {
    target = "LegacyJunosLoader5"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "SendLine"
      line = "set netmask=255.255.255.0"
    }
  }
}

state "LegacyJunosLoader5" {
  transition {
    target = "LegacyJunosLoader6"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "Send"
      text = "set gatewayip="
    }
    action {
      type = "SendConfigValue"
      key  = "tftp_server_ip"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}


state "LegacyJunosLoader6" {
  transition {
    target = "LegacyJunosLoader7"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "Send"
      text = "set serverip="
    }
    action {
      type = "SendConfigValue"
      key  = "tftp_server_ip"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "LegacyJunosLoader7" {
  transition {
    target = "WaitReimageFinish"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "Send"
      text = "install --format tftp://"
    }
    action {
      type = "SendConfigValue"
      key  = "tftp_server_ip"
    }
    action {
      type = "Send"
      text = "/"
    }
    action {
      type = "SendConfigValue"
      key  = "tftp_server_file"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
}

state "WaitReimageFinish" {
  transition {
    target = "JunosLogin"
    trigger {
      type = "string"
      string = "login:"
    }
    action {
      type = "SendLine"
      line = "root"
    }
  }
}

state "JunosLogin" {
  transition {
    target = "JunosHappyCli"
    trigger {
      type   = "string"
      string = "root@:RE:0%"
    }
    action {
      type = "SendLine"
      line = "sleep 30; cli"
    }
  }
}

state "JunosHappyCli" {
  transition {
    target = "JunosHappyCli"
    trigger {
      type = "string"
      string = "Retry connection attempts ? [yes,no] (yes)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "JunosPoweroffConfirm"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "SendLine"
      line = "request system power-off at now"
    }
  }
}


state "JunosPoweroffConfirm" {
  transition {
    target = "JunosWaitForPoweroff"
    trigger {
      type   = "string"
      string = "[yes,no] (no)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "JunosPoweroffConfirm"
    trigger {
      type   = "string"
      string = "error: command is not valid"
    }
    action {
      type = "SendLine"
      line = "request system halt at now"
    }
  }
}

state "JunosWaitForPoweroff" {
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "has halted."
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "acpi0: Powering system off"
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "Rebooting..."
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "reboot: Power down"
    }
  }
}

