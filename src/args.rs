use clap::{Arg, Command};

pub fn define_command_line_options(app: Command<'_>) -> Command<'_> {
    app.subcommand(
        Command::new("install")
            .about("Install shadowsocks")
            .arg(
                Arg::new("TYPE")
                    .default_value("rust")
                    .possible_values(["rust", "libev"])
                    .help("Shadowsocks installation type"),
            )
            .arg(
                Arg::new("SERVER_PORT")
                    .long("port")
                    .required(true)
                    .takes_value(true)
                    .validator(|p| p.parse::<i32>())
                    .help("Server port"),
            )
            .arg(
                Arg::new("SERVER_PASSWORD")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("Server password"),
            )
            .arg(
                Arg::new("CIPHER")
                    .long("cipher")
                    .default_value("aes-256-gcm")
                    .possible_values(["aes-256-gcm", "chacha20-ietf-poly1305", "aes-128-gcm"])
                    .help("AEAD cipher"),
            ),
    )
    .subcommand(
        Command::new("undo").arg(
            Arg::new("TYPE")
                .default_value("rust")
                .possible_values(["rust", "libev"])
                .help("Type of installation to undo"),
        ),
    )
}
