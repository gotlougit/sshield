# sshield

An experimental, opinionated drop-in `ssh-agent` replacement written in Rust using [`russh`](https://github.com/warp-tech/russh)
with the aim of being safer and more secure due to sandboxing sensitive data.

Right now, we are able to use it as an SSH agent, albeit with some degree of manual setup, 
which will be automated

It can:

- Import private OpenSSH keys from keyfiles into the database or generate new ones

- Act as SSH agent

- Create an authentication check whenever `ssh` client request key for a certain operation 

- Secure all key data through encryption of the database (using [SQLCipher](https://github.com/sqlcipher/sqlcipher))

- Use the OS keyring to store your database password (disabled by default) so it unlocks on login

To-dos:

- Allow easy exporting of key data to the OpenSSH format

- Lock the key data when user logs out

- Use [Landlock](https://docs.kernel.org/security/landlock.html) or [seccomp](https://en.wikipedia.org/wiki/Seccomp) based sandboxing to drop all privileges down to 
the bare minimum in order to protect the running process from various exploits

Nice-to-haves:

- Store key data on remote locations with different authentication schemes

- Store key data in different formats

- Configure database and socket location

## Usage

The best way is to use Nix and [home-manager](https://github.com/nix-community/home-manager). This way, a hardened user systemd service will be set up that runs on login.
You can configure sshield using Nix for greater flexibility.

The provided Home Manager module will also add the program to your user's PATH, so it can be invoked from the command line.

For a flake-based NixOS config, add this repo to your inputs:

```
inputs.sshield.url = "github:gotlougit/sshield";
inputs.sshield.inputs.nixpkgs.follows = "nixpkgs";
```

and write the following in your Home Manager config:

```
{ inputs, ... }:
{
  imports = [ inputs.sshield.hmModule ];
  programs.sshield = {
    enable = true;
    settings = {
      # Write your config here
      database = "/home/user/.sshield.db";
      prompt = 60;
      keyring = true;
    };
  };
}
```

This does all the hard work for you! You now have a hardened SSH agent using encrypted SSH keys
that unlocks the database on login using the keyring that comes with your desktop environment
(if any).
