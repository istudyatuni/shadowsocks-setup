# Shadowsocks setup

Helper to setup [shadowsocks](https://shadowsocks.org) server

## Usage

Download static-linked build [here](https://github.com/istudyatuni/shadowsocks-setup/releases) to the server and unpack.

Install [`shadowsocks-rust`](https://github.com/shadowsocks/shadowsocks-rust):

```bash
sssetup install --port <port> --password <password>
# explicitly specify the type
sssetup install rust --port <port> --password <password>
# specify AEAD cipher
sssetup install --port <port> --password <password> --cipher <cipher>
```

Undo installation:

```bash
sssetup undo
# explicitly specify the type
sssetup undo rust
```
