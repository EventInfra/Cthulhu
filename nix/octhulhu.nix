{
  config,
  lib,
  pkgs,
  ...
}:
let
  mCfg = config.services.cthulhu.mqtt;
  cfg = config.services.cthulhu.octhulhu-agent;
in
{
  config = lib.mkIf cfg.enable {
    environment.etc."cthulhu/octhulhu-agent.toml" = {
      source = pkgs.writers.writeTOML "octhulhu-agent.toml" {
        Heaven = {
          host = mCfg.host;
          port = mCfg.port;
          id = cfg.mqttId;
        };
        NetworkSerial = cfg.networkSerials;
        PortMapping = cfg.portMapping;
      };
    };

    systemd.services."octhulhu-agent" = {
      enable = true;
      after = [ "network.target" ];
      requires = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      description = "Cthulhu - Octhulhu Agent";
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/octhulhu-agent -c /etc/cthulhu/octhulhu-agent.toml daemon";
        Environment = "RUST_BACKTRACE=1";
        Restart = "on-failure";
        RestartSec = "5s";
        ConditionPathExists = "/etc/cthulhu/octhulhu-agent.toml";
      };
    };
  };
}
