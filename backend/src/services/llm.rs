use anyhow::Result;
use reqwest::Client;
use tracing::{info, warn};

use crate::models::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage};

pub struct LlmClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl LlmClient {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        LlmClient {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn summarize_readme_korean(&self, readme_content: &str, repo_name: &str) -> Result<Option<String>> {
        let url = format!("{}/chat/completions", self.base_url);

        let system_prompt = r#"You are a technical documentation summarizer. 
Your task is to summarize GitHub README content in Korean.
Focus on:
1. 프로젝트가 무엇인지 (What it does)
2. 주요 기능 (Key features)
3. 기술 스택 (Tech stack if mentioned)

Rules:
- Keep the summary under 200 characters
- Use Korean language only
- Be concise and informative
- Do not include markdown formatting
- Do not include links or code"#;

        let user_content = format!(
            "Summarize this README for the repository '{}' in Korean:\n\n{}",
            repo_name, readme_content
        );

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_content,
                },
            ],
            max_tokens: Some(300),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("LLM API error for {}: {} - {}", repo_name, status, error_text);
            return Ok(None);
        }

        let completion: ChatCompletionResponse = response.json().await?;
        
        if let Some(choice) = completion.choices.first() {
            let summary = choice.message.content.trim().to_string();
            info!("Generated Korean summary for {} ({} chars)", repo_name, summary.len());
            Ok(Some(summary))
        } else {
            warn!("No completion choices returned for {}", repo_name);
            Ok(None)
        }
    }
}
