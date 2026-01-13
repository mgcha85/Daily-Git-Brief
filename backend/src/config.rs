use anyhow::Result;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub oss_insight_base_url: String,
    pub github_api_url: String,
    pub github_token: Option<String>,
    pub deepseek_base_url: String,
    pub deepseek_api_key: String,
    pub deepseek_model: String,
    pub language_threshold: f64,
    pub database_path: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let github_token = env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty());

        Ok(Config {
            oss_insight_base_url: env::var("OSS_INSIGHT_BASE_URL")
                .unwrap_or_else(|_| "https://api.ossinsight.io".to_string()),
            github_api_url: "https://api.github.com".to_string(),
            github_token,
            deepseek_base_url: env::var("DEEPSEEK_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            deepseek_api_key: env::var("DEEPSEEK_API_KEY")
                .expect("DEEPSEEK_API_KEY must be set"),
            deepseek_model: env::var("DEEPSEEK_MODEL")
                .unwrap_or_else(|_| "deepseek-chat".to_string()),
            language_threshold: env::var("LANGUAGE_THRESHOLD")
                .unwrap_or_else(|_| "0.2".to_string())
                .parse()
                .unwrap_or(0.2),
            database_path: env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "./data/daily_git_brief.duckdb".to_string()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
        })
    }
}
