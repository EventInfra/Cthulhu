{
  config,
  lib,
  pkgs,
  ...
}:
let
  mCfg = config.services.cthulhu.mqtt;
  cfg = config.services.cthulhu.heaven;
in
{
  config = lib.mkIf cfg.enable {
    environment.etc."cthulhu/heaven.toml" = {
      source = pkgs.writers.writeTOML "heaven.toml" {
        MQTT = {
          host = mCfg.host;
          port = mCfg.port;
          id = cfg.mqttId;
        };
        Web = {
          listen_address = cfg.listenAddress;
        };
      };
    };

    systemd.services."cthulhu-heaven" = {
      enable = true;
      after = [ "network.target" ];
      requires = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      description = "Cthulhu - Heaven";
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/cthulhu-heaven -c /etc/cthulhu/heaven.toml";
        Environment = "RUST_BACKTRACE=1";
        Restart = "on-failure";
        RestartSec = "5s";
        ConditionPathExists = "/etc/cthulhu/heaven.toml";
      };
    };
  };
}
