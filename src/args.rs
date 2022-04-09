use clap::{Arg, Command};

pub fn define_command_line_options(app: Command<'_>) -> Command<'_> {
    app.arg(
        Arg::new("INSTALL_TYPE")
            .long("install")
            .default_value("rust")
            .possible_values(["rust", "libev"])
            .help("Shadowsocks installation type"),
    )
    .arg(
        Arg::new("SERVER_PORT")
            .long("port")
            .required(true)
            .help("Server port"),
    )
    .arg(
        Arg::new("SERVER_PASSWORD")
            .long("password")
            .required(true)
            .help("Server password"),
    )
    .arg(
        Arg::new("CIPHER")
            .long("cipher")
            .default_value("aes-256-gcm")
            .possible_values(["aes-256-gcm", "chacha20-ietf-poly1305", "aes-128-gcm"])
            .help("AEAD Cipher"),
    )
}
