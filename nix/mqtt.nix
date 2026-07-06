{ config, lib, ... }:
let
  cfg = config.services.cthulhu.mqtt;
in
{
  config = lib.mkIf cfg.configureServer {
    services.mosquitto = {
      enable = true;
      listeners = [
        {
          acl = [ "pattern readwrite #" ];
          omitPasswordAuth = true;
          settings.allow_anonymous = true;
          address = "::1";
        }
      ];
    };
  };
}
