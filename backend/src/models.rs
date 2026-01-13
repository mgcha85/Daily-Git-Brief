use serde::{Deserialize, Serialize};

// OSS Insight API Response
#[derive(Debug, Deserialize)]
pub struct OssInsightResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: OssInsightData,
}

#[derive(Debug, Deserialize)]
pub struct OssInsightData {
    pub columns: Vec<OssInsightColumn>,
    pub rows: Vec<OssInsightRow>,
}

#[derive(Debug, Deserialize)]
pub struct OssInsightColumn {
    pub col: String,
    pub data_type: String,
}

#[derive(Debug, Deserialize)]
pub struct OssInsightRow {
    pub repo_id: String,
    pub repo_name: String,
    pub primary_language: Option<String>,
    pub description: Option<String>,
    pub stars: Option<String>,
    pub forks: Option<String>,
    pub pull_requests: Option<String>,
    pub pushes: Option<String>,
    pub total_score: Option<String>,
    pub contributor_logins: Option<String>,
    pub collection_names: Option<String>,
}

// Database models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingRepo {
    pub date: String,
    pub repo_id: i64,
    pub repo_name: String,
    pub primary_language: Option<String>,
    pub description: Option<String>,
    pub korean_summary: Option<String>,
    pub stars: Option<i32>,
    pub forks: Option<i32>,
    pub pull_requests: Option<i32>,
    pub pushes: Option<i32>,
    pub total_score: Option<f64>,
    pub contributor_logins: Option<String>,
    pub collection_names: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLanguage {
    pub date: String,
    pub repo_id: i64,
    pub language: String,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageTrend {
    pub date: String,
    pub language: String,
    pub normalized_percentage: f64,
    pub repo_count: i32,
}

// API Response models
#[derive(Debug, Serialize)]
pub struct TrendingRepoResponse {
    pub rank: usize,
    pub repo_id: i64,
    pub repo_name: String,
    pub github_url: String,
    pub primary_language: Option<String>,
    pub languages: Vec<LanguageInfo>,
    pub description: Option<String>,
    pub korean_summary: Option<String>,
    pub stars: Option<i32>,
    pub forks: Option<i32>,
    pub total_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub language: String,
    pub percentage: f64,
}

// DeepSeek API models
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

// GitHub API models
#[derive(Debug, Deserialize)]
pub struct GitHubRepoInfo {
    pub default_branch: String,
}

pub type GitHubLanguages = std::collections::HashMap<String, u64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStatus {
    pub is_running: bool,
    pub message: String,
    pub current_count: usize,
    pub total_count: usize,
}
