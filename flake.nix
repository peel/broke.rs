{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci.url = "github:yusdacra/nix-cargo-integration";
    nci.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = inputs:
    inputs.nci.lib.makeOutputs {
      root = ./.;
      config = common: {
        shell.packages = [common.pkgs.rust-analyzer common.pkgs.rdkafka common.pkgs.protobuf] ++ common.pkgs.lib.optionals common.pkgs.stdenv.isDarwin [common.pkgs.darwin.apple_sdk.frameworks.Security common.pkgs.darwin.apple_sdk.frameworks.CoreFoundation common.pkgs.zlib];
        shell.env = [
          {name = "LD_LIBRARY_PATH"; value = "/run/current-system/sw/lib:$LD_LIBRARY_PATH";}
          {name = "PROTOC"; value = "${common.pkgs.protobuf}/bin/protoc";}
          {name = "LDFLAGS"; value = "-lgcc_eh"; }
        ];
      };
      pkgConfig = common:
        let overrides = rec {
          PROTOC = "${common.pkgs.protobuf}/bin/protoc";
          buildInputs = old: old ++ [ common.pkgs.protobuf common.pkgs.zlib common.pkgs.rdkafka ] ++ common.pkgs.lib.optionals common.pkgs.stdenv.isDarwin [common.pkgs.darwin.apple_sdk.frameworks.Security common.pkgs.darwin.apple_sdk.frameworks.CoreFoundation ];
          nativeBuildInputs = buildInputs;
          runtimeLibs = common.pkgs.protobuf;
        };
        in {
          default.depsOverrides.override = overrides;
          default.overrides.override = overrides;
      };
    };
}
