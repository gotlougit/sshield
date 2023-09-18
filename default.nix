{ lib, makeBinaryWrapper, rustPlatform, fetchFromGitHub, pkgs, cargo, pkg-config, openssl, libseccomp, sqlcipher, ... }:

rustPlatform.buildRustPackage rec {
  pname = "sshield";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "gotlougit";
    repo = pname;
    rev = "5ede7caa40effeeaddceee4308310da35b2048ac";
    sha256 = "sha256-YleacDAbha3R0Y31eDZBE07hQA+63bnIQ9GvWr5Sbes=";
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "extrasafe-0.2.0" = "sha256-jJIL/zD07eopvZO9h1X1XccTva4edurdANv//hPZwIw=";
      "russh-0.38.0-beta.1" = "sha256-j6jQtRBEDQmYo4XmEmri1BfgJOASIASaUTi29KU/9k8=";
    };
  };
  cargoSha256 = "";

  nativeBuildInputs = [ makeBinaryWrapper cargo rustPlatform.cargoSetupHook pkg-config ];
  buildInputs = [ openssl libseccomp sqlcipher ];
  wrapperPath = lib.makeBinPath ([
    pkgs.libsForQt5.kdialog
    pkgs.gnome.zenity
  ]);
  
  checkPhase = false;
  postFixup = ''
    wrapProgram $out/bin/sshield --prefix PATH : "${wrapperPath}"
  '';

  meta = with lib; {
    description = "A secure, drop-in, opinionated SSH agent replacement";
    homepage = "https://git.sr.ht/~gotlou/sshield";
    license = licenses.gpl2Plus;
  };
}
