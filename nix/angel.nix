{
  config,
  lib,
  pkgs,
  ...
}:
let
  mCfg = config.services.cthulhu.mqtt;
  cfg = config.services.cthulhu.angel;
  ports = builtins.attrNames cfg;

  mkServiceSpec = pName: {
    enable = true;
    after = [ "network.target" ];
    requires = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    description = "Cthulhu - Angel %i";
    serviceConfig = {
      Type = "simple";
      ExecStart = "${cfg.${pName}.package}/bin/cthulhu-angel -c /etc/cthulhu/angel/%i.toml";
      Environment = "RUST_BACKTRACE=1";
      Restart = "on-failure";
      RestartSec = "5s";
      ConditionPathExists = "/etc/cthulhu/angel/%i.toml";
    };
  };

  mkConfig =
    pName:
    let
      pCfg = cfg.${pName};

      tomlOpt = fName: value: if value == null then { } else { ${fName} = value; };
    in
    pkgs.writers.writeTOML "${pName}.toml" (
      {
        Heaven = {
          host = mCfg.host;
          port = mCfg.port;
          id = pName;
        };
      }
      // (tomlOpt "JobConfig" pCfg.jobConfig)
      // (tomlOpt "log_dir" pCfg.logDir)
      // (tomlOpt "log_level" pCfg.logLevel)
      // (tomlOpt "active_states" pCfg.activeStates)
      // (
        if pCfg.rawTcp == null then
          { }
        else
          {
            RawTCP = {
              endpoint = pCfg.rawTcp;
            };
          }
      )
      // (
        if pCfg.tty == null then
          { }
        else
          {
            TTY = pCfg.tty;
          }
      )
    );

in
{
  config = lib.mkIf (builtins.length ports != 0) {
    systemd.services = builtins.listToAttrs (
      map (n: {
        name = "cthulhu-angel@${n}";
        value = mkServiceSpec n;
      }) ports
    );

    environment.etc = builtins.listToAttrs (
      map (n: {
        name = "cthulhu/angel/${n}.toml";
        value = {
          source = mkConfig n;
        };
      }) ports
    );
  };
}
