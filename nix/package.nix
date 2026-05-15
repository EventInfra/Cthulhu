{
  crane,
  pkgs,
  pkg-config,
  udev,
  cthuluPackageName,
}:
let
  craneLib = crane.mkLib pkgs;
in
craneLib.buildPackage {
  src = ./..;
  pname = cthuluPackageName;
  cargoExtraArgs = "-p ${cthuluPackageName}";
  version = "0.1.0";
  buildInputs = [
    pkg-config
    udev
  ];
}
