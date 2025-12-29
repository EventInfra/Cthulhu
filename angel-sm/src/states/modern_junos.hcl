id = "modern_junos_wipe"

depends = [
  "common_junos_wipe"
]

state "SwitchDetect" {
  merge = "append"
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Booting from Flash A"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Primary BIOS version CDEN_P_EX1"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloaderMXAlt"
    trigger {
      type   = "string"
      string = "Primary BIOS version CICL_P_"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "U-Boot 2016"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Juniper U-Boot Script File"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "U-Boot 2021"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosWaitForBootloader"
    trigger {
      type   = "string"
      string = "Juniper Linux"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
}

state "ModernJunosWaitForBootloaderMXAlt" {
  transition {
    target = "ModernJunosBootloader1"
    trigger {
      type = "string"
      string = "seconds... (press Ctrl-C to interrupt)"
    }
    action {
      type = "Repeat"
      times = 10
      action {
        type = "SendControl"
        char = "c"
      }
    }
  }
}

state "ModernJunosWaitForBootloader" {
  transition {
    target = "ModernJunosWaitForBootloaderMXAlt"
    trigger {
      type   = "string"
      string = "Primary BIOS version CICL_P_"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "Juniper"
    }
  }
  transition {
    target = "ModernJunosEvo1"
    trigger {
      type = "string"
      string = "*Primary"
    }
    action {
      type = "SendControl"
      char = "n"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
  transition {
    target = "ModernJunosQFX1"
    trigger {
      type = "string"
      string = "The highlighted entry will be executed automatically"
    }
    action {
      type = "SendControl"
      char = "n"
    }
    action {
      type = "SendControl"
      char = "n"
    }
    action {
      type = "SendLine"
      line = ""
    }
  }
  transition {
    target = "ModernJunosBootloader1"
    trigger {
      type = "string"
      string = "seconds... (press Ctrl-C to interrupt)"
    }
    action {
      type = "Repeat"
      times = 10
      action {
        type = "SendControl"
        char = "c"
      }
    }
  }
}

state "ModernJunosEvo1" {
  transition {
    target = "ModernJunosEvo2"
    trigger {
      type = "string"
      string = "New password:"
    }
    action {
      type = "SendLine"
      line = "Password123!"
    }
  }
}

state "ModernJunosEvo2" {
  transition {
    target = "ModernJunosEvo3"
    trigger {
      type = "string"
      string = "Retype new password:"
    }
    action {
      type = "SendLine"
      line = "Password123!"
    }
  }
}

state "ModernJunosEvo3" {
  transition {
    target = "ModernJunosEvo4"
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

state "ModernJunosEvo4" {
  transition {
    target = "ModernJunosEvo4"
    trigger {
      type = "string"
      string = "Password:"
    }
    action {
      type = "SendLine"
      line = "Password123!"
    }
  }
  transition {
    target = "ModernJunosEvo4"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?:~#"
    }
    action {
      type = "SendLine"
      line = "cli"
    }
  }
  transition {
    target = "ModernJunosEvo4"
    trigger {
      type = "string"
      string = "Retry connection attempts ? [yes,no]"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "ModernJunosAnswerZeroize"
    trigger {
      type  = "regex"
      regex = "root(@[A-Za-z0-9\\-]+)?>"
    }
    action {
      type = "SendLine"
      line = "request system zeroize"
    }
  }
}

state "ModernJunosQFX1" {
  transition {
    target = "ModernJunosQFX2"
    trigger {
      type = "string"
      string = "Pick an option"
    }
    action {
      type = "SendLine"
      line = "1"
    }
  }
}

state "ModernJunosQFX2" {
  transition {
    target = "ModernJunosQFX2"
    trigger {
      type = "string"
      string = "Do you want to format"
    }
    action {
      type = "SendLine"
      line = "y"
    }
  }
  transition {
    target = "ModernJunosQFX2"
    trigger {
      type = "string"
      string = "Do you want to reimage"
    }
    action {
      type = "SendLine"
      line = "y"
    }
  }
  transition {
    target = "ModernJunosAwaitZeroizeFinish"
    trigger {
      type = "string"
      string = "Please confirm"
    }
    action {
      type = "SendLine"
      line = "1"
    }
  }
}

state "ModernJunosBootloader1" {
  transition {
    target = "ModernJunosBootloader2"
    trigger {
      type = "string"
      string = "Choice:"
    }
    action {
      type = "Send"
      text = "5"
    }
    action {
      type = "Flush"
    }
  }
}

state "ModernJunosBootloader2" {
  transition {
    target = "ModernJunosAwaitRecoveryShell"
    trigger {
      type = "string"
      string = "Choice:"
    }
    action {
      type = "Send"
      text = "2"
    }
    action {
      type = "Flush"
    }
  }
}

state "ModernJunosAwaitRecoveryShell" {
  transition {
    target = "ModernJunosAnswerZeroize"
    trigger {
      type = "regex"
      regex = "\\{[a-z0-9]+:0\\}"
    }
    action {
      type = "SendLine"
      line = "request system zeroize"
    }
  }
  transition {
    target = "ModernJunosAnswerZeroize"
    trigger {
      type = "string"
      string = "root>"
    }
    action {
      type = "SendLine"
      line = "request system zeroize"
    }
  }
}

state "ModernJunosAnswerZeroize" {
  transition {
    target = "ModernJunosAwaitZeroizeFinish"
    trigger {
      type = "string"
      string = "Erase all data, including configuration and log files?"
    }
    action {
      type = "SendLine"
      line = "yes"
    }
  }
  transition {
    target = "ModernJunosAnswerZeroize"
    trigger {
      type = "string"
      string = "ERROR: System not ready. Try again later."
    }
    action {
      type = "Delay"
      duration = 1.0
    }
    action {
      type = "SendLine"
      line = "request system zeroize"
    }
  }
}

state "ModernJunosAwaitZeroizeFinish" {
  transition {
    target = "ModernJunosAwaitZeroizeFinish2"
    trigger {
      type = "regex"
      regex = "FreeBSD/[AaipP]"
    }
  }
  transition {
    target = "ModernJunosAwaitZeroizeFinish2"
    trigger {
      type = "string"
      string = "mgd: warning: activating factory configuration"
    }
  }
}

state "ModernJunosAwaitZeroizeFinish2" {
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
