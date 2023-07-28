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

To-dos:

- Allow easy exporting of key data to the OpenSSH format

- Lock the key data when user logs out

- Use [Landlock](https://docs.kernel.org/security/landlock.html) or [seccomp](https://en.wikipedia.org/wiki/Seccomp) based sandboxing to drop all privileges down to 
the bare minimum in order to protect the running process from various exploits

Nice-to-haves:

- Store key data on remote locations with different authentication schemes

- Store key data in different formats

- Configure database and socket location