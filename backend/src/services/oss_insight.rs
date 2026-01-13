use anyhow::Result;
use reqwest::Client;
use tracing::info;

use crate::models::{OssInsightResponse, OssInsightRow};

pub struct OssInsightClient {
    client: Client,
    base_url: String,
}

impl OssInsightClient {
    pub fn new(base_url: &str) -> Self {
        OssInsightClient {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn get_trending_repos(&self) -> Result<Vec<OssInsightRow>> {
        let url = format!("{}/v1/trends/repos/", self.base_url);
        
        info!("Fetching trending repos from OSS Insight API");
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        let oss_response: OssInsightResponse = response.json().await?;
        
        info!("Fetched {} trending repos", oss_response.data.rows.len());
        
        Ok(oss_response.data.rows)
    }
}
