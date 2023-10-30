{ lib, makeBinaryWrapper, rustPlatform, fetchFromGitHub, pkgs, cargo, pkg-config, openssl, libseccomp, sqlcipher, ... }:

rustPlatform.buildRustPackage rec {
  pname = "sshield";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "gotlougit";
    repo = pname;
    rev = "7d5d51da4853401957eac693d51cbaff5c1ccd9d";
    sha256 = "sha256-HrzrJP65I2EfUbTFeNLLyplzJgsV6zee6nBajY3XJ5M=";
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
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
