{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };
  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    (
      {
        nixosModules = {
          cthulhu = ./nix;
        };
      }
      // flake-utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          craneLib = crane.mkLib pkgs;

          version = "0.1.0";

          cthulhu-heaven = craneLib.buildPackage {
            src = ./.;
            pname = "cthulhu-heaven";
            cargoExtraArgs = "-p cthulhu-heaven";
            inherit version;
          };

          cthulhu-angel = craneLib.buildPackage {
            src = ./.;
            pname = "cthulhu-angel";
            cargoExtraArgs = "-p cthulhu-angel";
            inherit version;
            buildInputs = with pkgs; [
              pkg-config
              udev
            ];
          };

          cthulhu-netbox = craneLib.buildPackage {
            src = ./.;
            pname = "cthulhu-netbox";
            cargoExtraArgs = "-p cthulhu-netbox";
            inherit version;
          };

          cthulhu-provision = craneLib.buildPackage {
            src = ./.;
            pname = "cthulhu-provision";
            cargoExtraArgs = "-p cthulhu-provision";
            inherit version;
          };

          octhulhu-agent = craneLib.buildPackage {
            src = ./.;
            pname = "octhulhu-agent";
            cargoExtraArgs = "-p octhulhu-agent";
            inherit version;
            buildInputs = with pkgs; [
              pkg-config
              udev
            ];
          };
        in
        {
          checks = {
            inherit
              cthulhu-heaven
              cthulhu-angel
              cthulhu-netbox
              cthulhu-provision
              octhulhu-agent
              ;
          };
          packages = {
            inherit
              cthulhu-heaven
              cthulhu-angel
              cthulhu-netbox
              cthulhu-provision
              octhulhu-agent
              ;
          };

        }
      )
    );
}
