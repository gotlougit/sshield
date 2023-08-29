{ lib, rustPlatform, fetchFromGitHub, pkgs, ... }:
let
  pname = "sshield";
  version = "0.1.0";
  src = fetchFromGitHub {
    owner = "gotlougit";
    repo = pname;
    rev = "98c411901d9423804d4ef43f6720ee47d458f323";
    sha256 = "sha256-zCC3oslVtMARJn4eKTxQTT80OYY26QZJP01c+xHdOzs=";
  };
  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
    outputHashes = {
      
    };
  };
in
rustPlatform.buildRustPackage {
  inherit cargoDeps;
  propagatedBuildInputs = [ pkgs.libsForQt5.kdialog ];
  cargoHash = "";

  meta = with lib; {
    description = "A secure, drop-in, opinionated SSH agent replacement";
    homepage = "https://git.sr.ht/~gotlou/sshield";
    license = licenses.gpl2Plus;
  };
}
