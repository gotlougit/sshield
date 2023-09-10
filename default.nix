{ lib, makeBinaryWrapper, rustPlatform, fetchFromGitHub, pkgs, cargo, pkg-config, openssl, libseccomp, sqlcipher, ... }:

rustPlatform.buildRustPackage rec {
  pname = "sshield";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "gotlougit";
    repo = pname;
    rev = "f72b14fada55d89716801199f3acc1be8944f0a6";
    sha256 = "sha256-5Fswkd7M7ijUZD6QXhu1wOt+c7eoyFVP6ilxHYoG39c=";
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
