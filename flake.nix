{
  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      mkShell = pkgs.mkShell.override { stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv; };
    in
    {
      devShells.${system}.default = mkShell {
        name = "rustdev";
        shellHook = ''
          export CARGO_HOME="$(realpath ./.localcargo)"
          export _ZO_DATA_DIR="$(realpath ./.localzoxide)"
        '';
        buildInputs = [
          pkgs.pkg-config
          pkgs.cargo
          pkgs.clippy
          pkgs.rustc
          pkgs.rustfmt
          pkgs.rust-analyzer
          pkgs.libseccomp
          pkgs.sqlcipher
          pkgs.gnome.zenity
          pkgs.libsForQt5.kdialog
          pkgs.gdb
        ];
      };
      packages.${system} = {
        default = pkgs.callPackage ./default.nix { };
      };
      hmModule = import ./home-manager.nix;
    };
}
