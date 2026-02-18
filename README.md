# Proxy setup helper

Helper to setup proxy on server. Currently it can install [shadowsocks](https://shadowsocks.org) and [xtls](https://xtls.github.io)

## Usage

Download static-linked build from [latest release](https://github.com/istudyatuni/shadowsocks-setup/releases/latest) to the server and unpack

### Shadowsocks

```bash
# install shadowsocks, input options interactively
sssetup ss install
# pass options from cli
sssetup ss install --port <port> --password <password> --cipher <cipher> --version <version>

# update shadowsocks
sssetup ss update
sssetup ss update --version <version>

# uninstall shadowsocks
sssetup ss uninstall
```

### Xray

```bash
# install xray. these are required options:
sssetup xray install --domain <domain> --zerossl-email <email>

# required options are omitted below for brevity

# add more users (default: 1)
sssetup xray install --add-users-count 5

# add users with specific uuid
sssetup xray install --add-user-id uuid1,uuid2

# set url for domain renewal
sssetup xray install --domain-renew-url <url>

# enable xray api
sssetup xray install --api
# set api port (default: 8080)
sssetup xray install --api --api-port 2345
```
