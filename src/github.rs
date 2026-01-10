use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

pub fn get_latest_release_tag(owner: &str, repo: &str) -> Result<String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");

    let resp = Client::new()
        .get(url)
        .header("accept", "application/vnd.github+json")
        .header("user-agent", "curl/8.17.0")
        .header("x-github-api-version", "2022-11-28")
        .send()
        .context("failed to send request for latest github release")?
        .error_for_status()?
        .text()
        .context("failed to get text of response")?;
    let Release { tag_name } =
        match serde_json::from_str::<Release>(&resp).context("failed to parse json") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("github api returns invalid json:\n{resp}");
                return Err(e);
            }
        };

    Ok(tag_name)
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}
