{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";
    nci.url = "github:yusdacra/nix-cargo-integration";
  };
  outputs = inputs:
    inputs.nci.lib.makeOutputs {
      root = ./.;
      config = common: {
        shell.packages = [common.pkgs.rust-analyzer] ++ common.pkgs.lib.optionals common.pkgs.stdenv.isDarwin [common.pkgs.darwin.apple_sdk.frameworks.Security common.pkgs.darwin.apple_sdk.frameworks.CoreFoundation common.pkgs.zlib];
      };
      pkgConfig = common: {
        default.depsOverrides.override.buildInputs = old: old ++ common.pkgs.lib.optionals common.pkgs.stdenv.isDarwin [common.pkgs.darwin.apple_sdk.frameworks.Security common.pkgs.darwin.apple_sdk.frameworks.CoreFoundation common.pkgs.zlib];
      };
    };
}
