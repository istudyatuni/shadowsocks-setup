use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::install::path_to_str;

#[derive(Debug, Serialize)]
pub struct XrayConfig {
    inbounds: Vec<InboundConfig>,

    #[serde(skip_serializing)]
    inbound_with_clients_index: usize,
}

#[derive(Debug, Serialize)]
struct InboundConfig {
    tag: String,
    settings: InboundConfigSettings,

    #[serde(flatten)]
    rest: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct InboundConfigSettings {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clients: Vec<Client>,

    #[serde(flatten)]
    rest: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct Client {
    pub id: String,
    flow: String,
}

impl XrayConfig {
    pub fn new(cert_dir: &Path) -> Result<Self> {
        let vless_inbound_rule = InboundConfig {
            tag: "vless".to_string(),
            settings: InboundConfigSettings {
                clients: vec![],
                rest: json!({
                    "decryption": "none",
                    "fallbacks": [{ "dest": 8000 }]
                }),
            },
            rest: json!({
                "port": 443,
                "protocol": "vless",
                "streamSettings": {
                    "network": "tcp",
                    "security": "tls",
                    "tlsSettings": {
                        "alpn": "http/1.1",
                        "certificates": [
                            {
                                "certificateFile": path_to_str(cert_dir.join("xray.crt"))?,
                                "keyFile": path_to_str(cert_dir.join("xray.key"))?,
                            }
                        ]
                    }
                }
            }),
        };

        Ok(Self {
            inbounds: vec![vless_inbound_rule],
            inbound_with_clients_index: 0,
        })
    }
    pub fn users(&self) -> &[Client] {
        &self.inbounds[self.inbound_with_clients_index]
            .settings
            .clients
    }
    fn users_mut(&mut self) -> &mut Vec<Client> {
        &mut self.inbounds[self.inbound_with_clients_index]
            .settings
            .clients
    }
    pub fn reserve_users_space(&mut self, count: usize) {
        self.users_mut().reserve(count);
    }
    pub fn add_users(&mut self, count: usize) -> &mut Self {
        self.reserve_users_space(count);
        for _ in 0..count {
            self.add_user();
        }
        self
    }
    fn add_user(&mut self) -> &mut Self {
        self.add_user_with_id(Uuid::new_v4().to_string().as_str())
    }
    pub fn add_user_with_id(&mut self, id: &str) -> &mut Self {
        self.users_mut().push(Client {
            id: id.to_string(),
            flow: "xtls-rprx-vision".to_string(),
        });
        self
    }
}
