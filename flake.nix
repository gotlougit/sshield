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
        '';
        buildInputs = [
          pkgs.pkgconfig
          pkgs.cargo
          pkgs.rustc
          pkgs.rustfmt
          pkgs.rust-analyzer
          pkgs.libseccomp
          pkgs.sqlcipher
          pkgs.gnome.zenity
          pkgs.libsForQt5.kdialog
        ];
      };
      packages.${system} = {
        default = pkgs.callPackage ./default.nix { };
      };
    };
}
