id = "legacy_junos_wipe"

depends = [
  "common_junos_wipe"
]

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
    target = "LegacyJunosWaitForRecoveryPrompt"
    trigger {
      type   = "string"
      string = "loader>"
    }
    action {
      type = "SendLine"
      line = ""
    }
    action {
      type = "SendLine"
      line = "boot -s"
    }
  }
}

state "LegacyJunosWaitForRecoveryPrompt" {
  transition {
    target = "LegacyJunosAwaitRecoveryShell"
    trigger {
      type   = "string"
      string = "Enter full pathname of shell or 'recovery' for root password recovery or RETURN for /bin/sh:"
    }
    action {
      type = "SendLine"
      line = "recovery"
    }
  }
  # EX3300s are known for their broken flash chips
  transition {
    target = "LegacyJunosWaitForRecoveryPrompt"
    trigger {
      type   = "string"
      string = "SCSI Status"
    }
    action {
      type = "AddDeviceInfo"
      flag = "SCSIErrors"
    }
  }
  # Boot loops have happened before.
  transition {
    target = "LegacyJunosUBoot1"
    trigger {
      type   = "string"
      string = "U-Boot 1.1"
    }
    action {
      type = "AddDeviceInfo"
      flag = "BootLoop"
    }
  }
}

state "LegacyJunosAwaitRecoveryShell" {
  transition {
    target = "LegacyJunosAnswerZeroize"
    trigger {
      type  = "regex"
      regex = "\\{[a-z0-9]+:0\\}"
    }
    action {
      type     = "Delay"
      duration = 3
    }
    action {
      type = "SendLine"
      line = "request system zeroize"
    }
  }
  # EX3300s are known for their broken flash chips
  transition {
    target = "LegacyJunosAwaitRecoveryShell"
    trigger {
      type   = "string"
      string = "SCSI Status"
    }
    action {
      type = "AddDeviceInfo"
      flag = "SCSIErrors"
    }
  }

  # Junipers like to have fsck issues if you unplug them wrong.
  transition {
    target = "LegacyJunosUBoot1"
    trigger {
      type   = "string"
      string = "error: filesystem consistency checks (fsck -p -y) failed"
    }
    action {
      type = "SendLine"
      line = "shell"
    }
    action {
      type = "Function"
      func = "FixFS"
    }
    action {
      type     = "Delay"
      duration = 3
    }
  }

  # OS Corruption related failures
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "CLI invocation failed"
    }
    action {
      type = "AddDeviceInfo"
      flag = "FailedToEnterSingleUserMode"
    }
    action {
      type = "AddDeviceInfo"
      flag = "OSCorruption"
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "continue, shell, abort, retry, or reboot"
    }
    action {
      type = "AddDeviceInfo"
      flag = "FailedToEnterSingleUserMode"
    }
    action {
      type = "AddDeviceInfo"
      flag = "OSCorruption"
    }
  }
}

state "LegacyJunosAnswerZeroize" {
  transition {
    target = "LegacyJunosAwaitZeroizeFinish"
    trigger {
      type   = "string"
      string = "[yes,no] (no)"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
}

state "LegacyJunosAwaitZeroizeFinish" {
  transition {
    target = "JunosLogin"
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
    target = "LegacyJunosAwaitZeroizeFinish"
    trigger {
      type   = "string"
      string = "Unable to load a kernel!"
    }
    action {
      type = "AddDeviceInfo"
      flag = "UnableToLoadAKernel"
    }
  }
  transition {
    target = "LegacyJunosAwaitZeroizeFinish"
    trigger {
      type   = "string"
      string = "can't load '/kernel'"
    }
    action {
      type = "AddDeviceInfo"
      flag = "UnableToLoadAKernel"
    }
  }
}
