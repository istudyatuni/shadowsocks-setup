#!/usr/bin/env sh
"HOME/.acme.sh/acme.sh" \
	--install-cert -d VAR_DOMAIN --ecc \
	--fullchain-file "VAR_HOME/xray-cert/xray.crt" \
	--key-file "VAR_HOME/xray-cert/xray.key"
echo renew done
chmod +r "VAR_HOME/xray-cert/xray.key"
sudo systemctl restart xray
