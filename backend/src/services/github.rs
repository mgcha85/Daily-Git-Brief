use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::models::{GitHubLanguages, GitHubRepoInfo, LanguageInfo};

pub struct GitHubClient {
    client: Client,
    api_url: String,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(api_url: &str, token: Option<String>) -> Self {
        GitHubClient {
            client: Client::new(),
            api_url: api_url.to_string(),
            token,
        }
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Daily-Git-Brief");
        
        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        req
    }

    pub async fn get_repo_languages(&self, repo_name: &str, threshold: f64) -> Result<Vec<LanguageInfo>> {
        let url = format!("{}/repos/{}/languages", self.api_url, repo_name);
        
        let response = self.build_request(&url).send().await?;
        
        if !response.status().is_success() {
            warn!("Failed to fetch languages for {}: {}", repo_name, response.status());
            return Ok(vec![]);
        }

        let languages: GitHubLanguages = response.json().await?;
        
        // Calculate total bytes
        let total: u64 = languages.values().sum();
        
        if total == 0 {
            return Ok(vec![]);
        }

        // Convert to percentages and filter by threshold
        let mut lang_info: Vec<LanguageInfo> = languages
            .into_iter()
            .map(|(lang, bytes)| {
                let percentage = (bytes as f64 / total as f64) * 100.0;
                LanguageInfo { language: lang, percentage }
            })
            .filter(|l| l.percentage >= (threshold * 100.0))
            .collect();

        // Sort by percentage descending
        lang_info.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

        info!("Found {} languages above {}% for {}", lang_info.len(), threshold * 100.0, repo_name);
        
        Ok(lang_info)
    }

    pub async fn get_readme(&self, repo_name: &str) -> Result<Option<String>> {
        // First, get the default branch
        let repo_url = format!("{}/repos/{}", self.api_url, repo_name);
        let repo_response = self.build_request(&repo_url).send().await?;
        
        if !repo_response.status().is_success() {
            warn!("Failed to fetch repo info for {}: {}", repo_name, repo_response.status());
            return Ok(None);
        }

        let repo_info: GitHubRepoInfo = repo_response.json().await?;
        let default_branch = repo_info.default_branch;

        // Fetch README from raw.githubusercontent.com
        let readme_urls = [
            format!("https://raw.githubusercontent.com/{}/{}/README.md", repo_name, default_branch),
            format!("https://raw.githubusercontent.com/{}/{}/readme.md", repo_name, default_branch),
            format!("https://raw.githubusercontent.com/{}/{}/Readme.md", repo_name, default_branch),
        ];

        for url in readme_urls {
            let response = self.client
                .get(&url)
                .header("User-Agent", "Daily-Git-Brief")
                .send()
                .await?;

            if response.status().is_success() {
                let content = response.text().await?;
                // Truncate if too long (limit to first 8000 chars for LLM context)
                // Use char_indices to find safe UTF-8 boundary
                let truncated = if content.len() > 8000 {
                    let mut end_idx = 8000;
                    // Find a valid char boundary
                    while !content.is_char_boundary(end_idx) && end_idx > 0 {
                        end_idx -= 1;
                    }
                    content[..end_idx].to_string()
                } else {
                    content
                };
                info!("Fetched README for {} ({} chars)", repo_name, truncated.len());
                return Ok(Some(truncated));
            }
        }

        warn!("No README found for {}", repo_name);
        Ok(None)
    }
}
