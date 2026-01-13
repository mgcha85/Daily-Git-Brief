use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::config::Config;
use crate::db::Database;
use crate::models::{TrendingRepo, RepoLanguage, LanguageTrend, CollectionStatus};
use crate::services::{OssInsightClient, GitHubClient, LlmClient};
use tokio::sync::broadcast;

pub struct DataCollector {
    oss_client: OssInsightClient,
    github_client: GitHubClient,
    llm_client: LlmClient,
    db: Database,
    language_threshold: f64,
}

impl DataCollector {
    pub fn new(config: &Config, db: Database) -> Self {
        DataCollector {
            oss_client: OssInsightClient::new(&config.oss_insight_base_url),
            github_client: GitHubClient::new(&config.github_api_url, config.github_token.clone()),
            llm_client: LlmClient::new(&config.deepseek_base_url, &config.deepseek_api_key, &config.deepseek_model),
            db,
            language_threshold: config.language_threshold,
        }
    }

    pub async fn collect(&self, progress_tx: Option<broadcast::Sender<CollectionStatus>>) -> Result<usize> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        info!("Starting data collection for {}", today);

        // Step 1: Fetch trending repos from OSS Insight
        let oss_repos = self.oss_client.get_trending_repos().await?;
        let total_repos = oss_repos.len();
        info!("Fetched {} repos from OSS Insight", total_repos);

        if let Some(tx) = &progress_tx {
            let _ = tx.send(CollectionStatus {
                is_running: true,
                message: format!("Fetched {} repos from OSS Insight", total_repos),
                current_count: 0,
                total_count: total_repos,
            });
        }

        let mut language_stats: HashMap<String, (f64, i32)> = HashMap::new();
        let mut collected_count = 0;

        // Get existing repo IDs that already have summaries (to skip)
        let existing_ids = self.db.get_existing_repo_ids(&today).unwrap_or_default();
        let skipped_count = existing_ids.len();
        if skipped_count > 0 {
            info!("Skipping {} repos that already have summaries", skipped_count);
        }

        for (i, oss_repo) in oss_repos.iter().enumerate() {
            let repo_id: i64 = oss_repo.repo_id.parse().unwrap_or(0);
            let repo_name = &oss_repo.repo_name;

            // Skip if already has summary for today
            if existing_ids.contains(&repo_id) {
                info!("Skipping {} (already has summary)", repo_name);
                continue;
            }

            // Step 2: Fetch README and generate Korean summary
            let korean_summary = match self.github_client.get_readme(repo_name).await {
                Ok(Some(readme)) => {
                    match self.llm_client.summarize_readme_korean(&readme, repo_name).await {
                        Ok(summary) => summary,
                        Err(e) => {
                            warn!("Failed to summarize README for {}: {}", repo_name, e);
                            None
                        }
                    }
                }
                Ok(None) => None,
                Err(e) => {
                    warn!("Failed to fetch README for {}: {}", repo_name, e);
                    None
                }
            };

            // Step 3: Fetch language statistics
            let languages = match self.github_client.get_repo_languages(repo_name, self.language_threshold).await {
                Ok(langs) => langs,
                Err(e) => {
                    warn!("Failed to fetch languages for {}: {}", repo_name, e);
                    vec![]
                }
            };

            // Save repo languages
            for lang in &languages {
                let repo_lang = RepoLanguage {
                    date: today.clone(),
                    repo_id,
                    language: lang.language.clone(),
                    percentage: lang.percentage,
                };
                if let Err(e) = self.db.save_repo_language(&repo_lang) {
                    warn!("Failed to save language for {}: {}", repo_name, e);
                }

                // Accumulate for daily trend
                let entry = language_stats.entry(lang.language.clone()).or_insert((0.0, 0));
                entry.0 += lang.percentage;
                entry.1 += 1;
            }

            // Save trending repo
            let trending_repo = TrendingRepo {
                date: today.clone(),
                repo_id,
                repo_name: repo_name.clone(),
                primary_language: oss_repo.primary_language.clone(),
                description: oss_repo.description.clone(),
                korean_summary,
                stars: oss_repo.stars.as_ref().and_then(|s| s.parse().ok()),
                forks: oss_repo.forks.as_ref().and_then(|s| s.parse().ok()),
                pull_requests: oss_repo.pull_requests.as_ref().and_then(|s| s.parse().ok()),
                pushes: oss_repo.pushes.as_ref().and_then(|s| s.parse().ok()),
                total_score: oss_repo.total_score.as_ref().and_then(|s| s.parse().ok()),
                contributor_logins: oss_repo.contributor_logins.clone(),
                collection_names: oss_repo.collection_names.clone(),
            };

            if let Err(e) = self.db.save_trending_repo(&trending_repo) {
                warn!("Failed to save trending repo {}: {}", repo_name, e);
            } else {
                collected_count += 1;
            }

            // Rate limiting: small delay between repos
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if let Some(tx) = &progress_tx {
                let _ = tx.send(CollectionStatus {
                    is_running: true,
                    message: format!("Processed {}", repo_name),
                    current_count: i + 1,
                    total_count: total_repos,
                });
            }
        }

        // Step 4: Calculate and save daily language trends (normalized)
        let total_percentage: f64 = language_stats.values().map(|(p, _)| p).sum();
        
        if total_percentage > 0.0 {
            for (language, (sum_percentage, repo_count)) in &language_stats {
                let normalized = (sum_percentage / total_percentage) * 100.0;
                let trend = LanguageTrend {
                    date: today.clone(),
                    language: language.clone(),
                    normalized_percentage: normalized,
                    repo_count: *repo_count,
                };
                if let Err(e) = self.db.save_language_trend(&trend) {
                    warn!("Failed to save language trend for {}: {}", language, e);
                }
            }
            info!("Saved {} language trends", language_stats.len());
        }

        info!("Data collection complete. Collected {} repos.", collected_count);
        
        if let Some(tx) = &progress_tx {
            let _ = tx.send(CollectionStatus {
                is_running: false,
                message: format!("Collection complete. Collected {} repos.", collected_count),
                current_count: total_repos,
                total_count: total_repos,
            });
        }
        
        Ok(collected_count)
    }
}
