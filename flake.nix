{
  description = "Rust dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };
  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        naersk' = pkgs.callPackage naersk { };
      in
      rec {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            (pkgs.fenix.stable.withComponents [
              "rustc"
              "cargo"
              "rust-analyzer"
              "clippy"
              "rustfmt"
            ])
            openssl
          ];
          nativeBuildInputs = with pkgs; [
            openssl
            pkg-config
          ];
        };
      }
    );
}
