{ config, lib, makeBinaryWrapper, rustPlatform, fetchFromGitHub, pkgs, cargo, pkg-config, openssl, libseccomp, sqlcipher, ... }:

with lib;

let
  cfg = config.programs.sshield;
  tomlFormat = pkgs.formats.toml { };
in {

  options.programs.sshield = {
    enable = mkEnableOption "sshield, secure SSH agent";

    extraPackages = mkOption {
      type = with types; listOf package;
      default = [ ];
      example = literalExpression "[ ]";
      description = "Extra packages available to sshield.";
    };

    settings = mkOption {
      type = tomlFormat.type;
      default = { };
      example = literalExpression ''
        {
          database = "/home/user/.ssh.db3";
          prompt = 60;
          keyring = true;
        }
      '';
      description = ''
        Configuration written to
        {file}`$XDG_CONFIG_HOME/sshield/sshield.toml`.
      '';
    };

  };

  config = mkIf cfg.enable {
    home.packages = [ (pkgs.callPackage ./default.nix { }) ];  
    home.sessionVariablesExtra = ''
      if [[ -z "$SSH_AUTH_SOCK" ]]; then
        export SSH_AUTH_SOCK=$XDG_RUNTIME_DIR/ssh-agent
      fi
    '';
    
    systemd.user.services.sshield = {
      Unit = {
        Description = "Secure SSH agent written in Rust";
      };
      Install = {
        WantedBy = [ "default.target" ];
      };
      Service = {
        ExecStart = "${pkgs.writeShellScript "sshield-serve" ''
          #!/run/current-system/sw/bin/bash
          sshield serve
        ''}";
        ExecStop = "${pkgs.writeShellScript "sshield-stop" ''
          #!/run/current-system/sw/bin/bash
          rm "$XDG_RUNTIME_DIR"/ssh-agent
        ''}";
        LockPersonality = true;
        PrivateNetwork = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        PrivateMounts = true;
        PrivateTmp = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        RestrictAddressFamilies = [ "AF_UNIX" ];
        SystemCallArchitectures = "native";
        PrivateDevices = true;
        SystemCallFilter = [ "@known" "~@clock" "~@cpu-emulation" "~@raw-io" "~@reboot" "~@mount" "~@obsolete" "~@swap" "~@debug" "~@keyring" "~@pkey" "~@chown" ];
      };
    };

    xdg.configFile = let
      settings = {
        "sshield/sshield.toml" = mkIf (cfg.settings != { }) {
          source = tomlFormat.generate "sshield-config" cfg.settings;
        };
      };
    in settings;
  };
}
