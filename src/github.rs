use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

pub fn get_latest_release_tag(owner: &str, repo: &str) -> Result<String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");

    let Release { tag_name } = Client::new()
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .context("failed to send request for latest github release")?
        .json()
        .context("failed to parse github response")?;

    Ok(tag_name)
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}
