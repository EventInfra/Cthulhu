{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.cthulhu.provision;
in
{
  config = lib.mkIf cfg.enable {
    environment.etc."cthulhu/provision.toml" = {
      source = pkgs.writers.writeTOML "provision.toml" {
        config_server = cfg.configServer;
        ntp_server = cfg.ntpServer;

        Web = {
          listen_address = cfg.listenAddress;
        };

        ModelOSMapping = cfg.modelOSMappings;
      };
    };

    systemd.services."cthulhu-provision" = {
      enable = true;
      after = [ "network.target" ];
      requires = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      description = "Cthulhu - Provision";
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/cthulhu-provision -c /etc/cthulhu/provision.toml";
        Environment = "RUST_BACKTRACE=1";
        Restart = "on-failure";
        RestartSec = "5s";
        ConditionPathExists = "/etc/cthulhu/provision.toml";
      };
    };
  };
}
