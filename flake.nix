{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
  };
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];

      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      flake.nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        {
          imports = [
            ./nix/default.nix
          ];
        };
      perSystem =
        {
          config,
          pkgs,
          lib,
          ...
        }:
        {
          overlayAttrs = {
            inherit (config.packages)
              cthulhu-heaven
              cthulhu-angel
              cthulhu-netbox
              cthulhu-provision
              octhulhu-agent
              ;
          };
          packages =
            let
              call =
                pName:
                pkgs.callPackage ./nix/package.nix {
                  cthuluPackageName = pName;
                  crane = inputs.crane;
                };
              packages = [
                "cthulhu-heaven"
                "cthulhu-angel"
                "cthulhu-netbox"
                "cthulhu-provision"
                "octhulhu-agent"
              ];
            in
            builtins.listToAttrs (
              map (n: {
                name = n;
                value = call n;
              }) packages
            );

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              graphviz
              cargo
            ];
          };
        };
    };
}
