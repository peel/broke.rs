{
  description = "Broker comparison testing utils";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        #   .override {
        #    targets = [ "aarch64-unknown-linux-musl" ];
        # };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = [pkgs.protobuf pkgs.zlib pkgs.rdkafka pkgs.openssl pkgs.pkgconfig] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.CoreFoundation];
          # CARGO_BUILD_TARGET = "aarch64-unknown-linux-musl";
          # CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        };
      in
        {
          checks = {
            inherit default;
          };

          packages = {
            inherit default;
            docker = pkgs.dockerTools.buildLayeredImage {
              name = "peel/stream-operator";
              created = "now";
              tag = "latest";
              contents = [ default ];
              config.Cmd = [ "/bin/nats" ];
            };
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = builtins.attrValues self.checks;
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
              rust-analyzer
              alejandra
            ];
          };
        });
}
