# Shadowsocks setup

Helper to setup [shadowsocks](//shadowsocks.org) server

## Usage

Download static-linked build [here](https://github.com/istudyatuni/shadowsocks-setup/releases) to the server and unpack. Now requires Ubuntu and can install only [shadowsocks-rust](https://github.com/shadowsocks/shadowsocks-rust).

Command-line options:

```
USAGE:
    sssetup [OPTIONS] --port <SERVER_PORT> --password <SERVER_PASSWORD>

OPTIONS:
        --cipher <CIPHER>               AEAD Cipher [default: aes-256-gcm] [possible values: aes-256-gcm, chacha20-ietf-poly1305, aes-128-gcm]
    -h, --help                          Print help information
        --install <INSTALL_TYPE>        Shadowsocks installation type [default: rust] [possible values: rust, libev]
        --password <SERVER_PASSWORD>    Server password
        --port <SERVER_PORT>            Server port
```
