# sshield

An experimental, opinionated drop-in `ssh-agent` replacement written in Rust using [`russh`](https://github.com/warp-tech/russh)
with the aim of being safer and more secure due to sandboxing sensitive data.

Aims:

- Secure all key data

- Create an authentication check whenever `ssh` client request key for a certain operation 

- Allow easy importing and exporting of key data to and from the OpenSSH format

- Lock the key data when user logs out

Nice-to-haves:

- Store key data on remote locations with different authentication schemes

- Store key data in different formats