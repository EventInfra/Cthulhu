id = "hp_wipe"

state "SwitchDetect" {
  merge = "append"
  transition {
    target = "HPWaitForBootloader1"
    trigger {
      type   = "string"
      string = "ROM information:"
    }
    action {
      type   = "AddDeviceInfo"
      Vendor = "HP"
    }
  }
}

state "HPWaitForBootloader1" {
  transition {
    target = "HPWaitForBootloader2"
    trigger {
      type   = "string"
      string = "0. Monitor ROM Console"
    }
  }
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "Select profile"
    }
    action {
      type = "AddDeviceInfo"
      flag = "Aborted"
    }
  }
}


state "HPWaitForBootloader2" {
  transition {
    target = "HPEnterBootloader"
    trigger {
      type   = "string"
      string = "Select profile"
    }
    action {
      type = "Function"
      func = "CaptureHPOSVersionBanner"
    }
    action {
      type = "Send"
      text = "0"
    }
    action {
      type = "Flush"
    }
  }
}

state "HPEnterBootloader" {
  transition {
    target = "HPCaptureModel"
    trigger {
      type   = "string"
      string = "=>"
    }
    action {
      type = "SendLine"
      line = "id"
    }
  }
}

state "HPCaptureModel" {
  transition {
    target = "HPCaptureSerial"
    trigger {
      type   = "string"
      string = "=>"
    }
    action {
      type = "Function"
      func = "CaptureHPSwitchModel"
    }
    action {
      type = "SendLine"
      line = "cat cfa0/idprofile01.inf"
    }
  }
}

state "HPCaptureSerial" {
  transition {
    target = "HPEraseConfirm"
    trigger {
      type   = "string"
      string = "=>"
    }
    action {
      type = "Function"
      func = "CaptureHPSwitchSerial"
    }
    action {
      type = "SendLine"
      line = "erase-all"
    }
  }
}

state "HPEraseConfirm" {
  transition {
    target = "HPWaitEraseComplete"
    trigger {
      type   = "string"
      string = "Continue (y/n)?"
    }
    action {
      type = "SendLine"
      line = "y"
    }
  }
}

state "HPWaitEraseComplete" {
  transition {
    target = "EndJob"
    trigger {
      type   = "string"
      string = "Waiting for Speed Sense."
    }
  }
}
