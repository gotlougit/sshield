{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        mkShell = pkgs.mkShell.override { stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv; };
      in with pkgs; rec {
        devShell = mkShell rec {
          buildInputs = [
            pkgconfig
            openssl.dev
            gnome.zenity
            libsForQt5.kdialog
            cargo
            rustc
            rustfmt
            rust-analyzer
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
}