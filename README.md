# Shadowsocks setup

Helper to setup [shadowsocks](https://shadowsocks.org) server

## Usage

Download static-linked build from [latest release](https://github.com/istudyatuni/shadowsocks-setup/releases/latest) to the server and unpack

```bash
# install, input options interactively
sssetup install
# pass options from cli
sssetup install --port <port> --password <password> --cipher <cipher> --version <version>

# update shadowsocks
sssetup update
sssetup update --version <version>

# undo installation
sssetup undo
```
