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
        };

        craneLib = (crane.mkLib pkgs);

        default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = [pkgs.protobuf pkgs.zlib pkgs.rdkafka pkgs.openssl pkgs.pkg-config] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.CoreFoundation];
        };
      in
        {
          checks = {
            inherit default;
          };

          packages = {
            inherit default;
            docker = pkgs.dockerTools.buildLayeredImage {
              name = "peelsky/stream-operator";
              created = "now";
              tag = "latest";
              contents = [ default ];
              config.Cmd = [ "/bin/nats" ];
            };
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = builtins.attrValues self.checks;
            nativeBuildInputs = with pkgs; [
              openssl
              pkg-config
              zlib
              protobuf
              cargo
              rustc
              rust-analyzer
              rustfmt
              alejandra
              natscli
              (pkgs.wrapHelm pkgs.kubernetes-helm { plugins = [ pkgs.kubernetes-helmPlugins.helm-diff ]; })
              helmfile
       	      gnuplot
              (google-cloud-sdk.withExtraComponents [google-cloud-sdk.components.gke-gcloud-auth-plugin])
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.CoreFoundation];
          };
        });
}
