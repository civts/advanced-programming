{
  description = "Rust develpoment environment";

  inputs = {
    nixpkgs.url = "github:nixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
          };
        };
      in
      {
        devShell = with pkgs; mkShell rec {
          #ENV_VARIABLE_1 = "test";
          nativeBuildInputs = [
            pkg-config
            stdenv.cc
            crate2nix
          ];
          propagatedBuildInputs = [ stdenv.cc ];
          buildInputs = [
            rustc
            rustfmt
            cargo
            clippy
            rustup
            gcc
            (vscode-with-extensions.override {
              vscode = vscodium;
              vscodeExtensions = with vscode-extensions; [
                jnoortheen.nix-ide
                matklad.rust-analyzer
                vadimcn.vscode-lldb
                bungcip.better-toml
              ] ++ pkgs.vscode-utils.extensionsFromVscodeMarketplace [
                {
                  name = "codegeex";
                  publisher = "aminer";
                  version = "1.0.6";
                  sha256 = "sha256-q8HSFZRhwZv5zApHsVoyKGqZsDDyUqjxv/qwGAuOE0c=";
                }
                {
                  name = "material-icon-theme";
                  publisher = "PKief";
                  version = "4.21.0";
                  sha256 = "sha256-EwJ4zGDdEak9fBAnn5pfuAU/+ONYWzl7Q6OMyc6mcZU=";
                }
              ];
            })
          ];
        };
      });
}
