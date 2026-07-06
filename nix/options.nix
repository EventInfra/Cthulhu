{
  lib,
  pkgs,
  ...
}:
{
  options.services.cthulhu = lib.mkOption {
    type = lib.types.submodule {
      options = {
        mqtt = lib.mkOption {
          type = lib.types.submodule {
            options = {
              configureServer = lib.mkEnableOption "";
              host = lib.mkOption {
                default = "localhost";
                type = lib.types.str;
              };
              port = lib.mkOption {
                default = 1883;
                type = lib.types.port;
              };
            };
          };
        };

        heaven = {
          enable = lib.mkEnableOption "";
          package = lib.mkOption {
            default = pkgs.cthulhu-heaven;
            type = lib.types.package;
          };
          listenAddress = lib.mkOption {
            default = "[::]:4040";
            type = lib.types.str;
          };
          mqttId = lib.mkOption {
            default = "heaven";
            type = lib.types.nullOr lib.types.str;
          };
        };

        angel = lib.mkOption {
          type = lib.types.attrsOf (
            lib.types.submodule {
              options = {
                package = lib.mkOption {
                  default = pkgs.cthulhu-angel;
                  type = lib.types.package;
                };
                mqttId = lib.mkOption {
                  type = lib.types.str;
                };
                rawTcp = lib.mkOption {
                  default = null;
                  type = lib.types.nullOr lib.types.str;
                };
                tty = lib.mkOption {
                  default = null;
                  type = lib.types.nullOr (
                    lib.types.submodule {
                      options = {
                        path = lib.mkOption {
                          type = lib.types.str;
                        };
                        baudrate = lib.mkOption {
                          type = lib.types.str;
                        };
                      };
                    }
                  );
                };
                logLevel = lib.mkOption {
                  default = null;
                  type = lib.types.nullOr lib.types.str;
                };
                logDir = lib.mkOption {
                  default = "/var/log/cthulhu";
                  type = lib.types.str;
                };
                activeStates = lib.mkOption {
                  default = [ "wipe" ];
                  type = lib.types.listOf lib.types.str;
                };
                jobConfig = lib.mkOption {
                  default = { };
                  type = lib.types.attrsOf lib.types.str;
                };
              };
            }
          );
        };

        octhulhu-agent = lib.mkOption {
          type = lib.types.submodule {
            options = {
              enable = lib.mkEnableOption "";

              package = lib.mkOption {
                default = pkgs.octhulhu-agent;
                type = lib.types.package;
              };

              mqttId = lib.mkOption {
                type = lib.types.str;
                default = "O1";
              };

              networkSerials = lib.mkOption {
                type = lib.types.listOf (
                  lib.types.submodule {
                    options = {
                      host = lib.mkOption {
                        type = lib.types.str;
                      };
                      port = lib.mkOption {
                        type = lib.types.port;
                      };
                    };
                  }
                );
              };

              portMapping = lib.mkOption {
                type = lib.types.attrsOf (lib.types.listOf lib.types.str);
              };
            };
          };
        };

        provision = lib.mkOption {
          type = lib.types.submodule {
            options = {
              enable = lib.mkEnableOption "";

              package = lib.mkOption {
                default = pkgs.cthulhu-provision;
                type = lib.types.package;
              };

              listenAddress = lib.mkOption {
                default = "[::]:5050";
                type = lib.types.str;
              };

              configServer = lib.mkOption {
                type = lib.types.str;
              };

              ntpServer = lib.mkOption {
                type = lib.types.str;
              };

              modelOSMappings = lib.mkOption {
                type = lib.types.listOf (
                  lib.types.submodule {
                    vendor = lib.mkOption {
                      type = lib.types.str;
                    };
                    model = lib.mkOption {
                      type = lib.types.str;
                    };
                    target_version = lib.mkOption {
                      type = lib.types.str;
                    };
                    os_image = lib.mkOption {
                      type = lib.types.path;
                    };
                  }
                );
              };
            };
          };
        };
      };
    };
  };
}
