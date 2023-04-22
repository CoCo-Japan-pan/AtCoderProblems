use anyhow::{anyhow, Context, Result};

use reqwest::Client;
use serde::de::DeserializeOwned;

pub(crate) async fn get_html(url: &str, client: &Client) -> Result<(String, reqwest::StatusCode)> {
    let response = client
        .get(url)
        .header("accept", "text/html")
        .send()
        .await
        .map_err(|e| anyhow!("Connection error: {:?}", e))?;
    let status = response.status();
    if !status.is_success() {
        log::error!("{:?}", response);
    }
    let body = response
        .text()
        .await
        .map_err(|e| anyhow!("Failed to parse HTTP body: {:?}", e))?;
    Ok((body, status))
}

pub(crate) async fn get_json<T: DeserializeOwned>(url: &str, client: &Client) -> Result<T> {
    client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .with_context(|| format!("Failed to get json from {}", url))?
        .json::<T>()
        .await
        .with_context(|| format!("Failed to parse json from {}", url))
}

pub trait Problem {
    fn url(&self) -> String;
}
