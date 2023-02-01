{
  description = "Broker comparison testing utils";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    devenv.url = "github:cachix/devenv";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    crane,
    flake-utils,
    devenv,
    fenix,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      craneLib = crane.lib.${system}.overrideToolchain fenix.packages.${system}.minimal.toolchain;

      default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
        buildInputs = [pkgs.protobuf pkgs.zlib pkgs.rdkafka pkgs.openssl pkgs.pkg-config] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.CoreFoundation pkgs.libiconv];
        nativeBuildInputs = [pkgs.protobuf pkgs.zlib pkgs.zstd pkgs.rdkafka pkgs.openssl pkgs.pkg-config] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.CoreFoundation pkgs.libiconv];
      };
    in {
      checks = {
        inherit default;
      };

      packages = {
        inherit default;
        docker = pkgs.dockerTools.buildLayeredImage {
          name = "peelsky/soma";
          created = "now";
          tag = "latest";
          contents = [default];
          config.Entrypoint = ["/bin/soma"];
          config.Cmd = ["nats"];
        };
      };

      devShells.default = devenv.lib.mkShell {
        # inputs = builtins.attrValues self.checks;
        inherit inputs pkgs;
        modules = [
          {
            env.GOOGLE_APPLICATION_CREDENTIALS = "$PWD/gcp-sandbox-token.json";
            difftastic.enable = true;
            languages.rust.enable = true;
            pre-commit.hooks = {
              alejandra.enable = true;
              deadnix.enable = true;
              clippy.enable = true;
              rustfmt.enable = true;
            };
            enterShell = ''
            '';
            packages = with pkgs;
              [
                openssl
                pkg-config
                zlib
                zstd
                protobuf
                libiconv
                cargo
                clippy
                rustc
                rust-analyzer
                rustfmt
                alejandra
                natscli
                rdkafka
                (pkgs.wrapHelm pkgs.kubernetes-helm {plugins = [pkgs.kubernetes-helmPlugins.helm-diff];})
                helmfile
                gnuplot
                (google-cloud-sdk.withExtraComponents [google-cloud-sdk.components.gke-gcloud-auth-plugin])
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.CoreFoundation];
          }
        ];
      };
    });
}
