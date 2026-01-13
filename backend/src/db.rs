use anyhow::Result;
use duckdb::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::models::{TrendingRepo, RepoLanguage, LanguageTrend};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS trending_repos (
                date DATE NOT NULL,
                repo_id BIGINT NOT NULL,
                repo_name VARCHAR NOT NULL,
                primary_language VARCHAR,
                description VARCHAR,
                korean_summary VARCHAR,
                stars INTEGER,
                forks INTEGER,
                pull_requests INTEGER,
                pushes INTEGER,
                total_score DOUBLE,
                contributor_logins VARCHAR,
                collection_names VARCHAR,
                PRIMARY KEY (date, repo_id)
            );

            CREATE TABLE IF NOT EXISTS repo_languages (
                date DATE NOT NULL,
                repo_id BIGINT NOT NULL,
                language VARCHAR NOT NULL,
                percentage DOUBLE NOT NULL,
                PRIMARY KEY (date, repo_id, language)
            );

            CREATE TABLE IF NOT EXISTS daily_language_trends (
                date DATE NOT NULL,
                language VARCHAR NOT NULL,
                normalized_percentage DOUBLE NOT NULL,
                repo_count INTEGER NOT NULL,
                PRIMARY KEY (date, language)
            );

            CREATE INDEX IF NOT EXISTS idx_trending_date ON trending_repos(date);
            CREATE INDEX IF NOT EXISTS idx_languages_date ON repo_languages(date);
            CREATE INDEX IF NOT EXISTS idx_trends_date ON daily_language_trends(date);
        "#)?;

        Ok(())
    }

    pub fn save_trending_repo(&self, repo: &TrendingRepo) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            r#"INSERT INTO trending_repos 
               (date, repo_id, repo_name, primary_language, description, korean_summary, 
                stars, forks, pull_requests, pushes, total_score, contributor_logins, collection_names)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT (date, repo_id) DO UPDATE SET
                   repo_name = excluded.repo_name,
                   primary_language = excluded.primary_language,
                   description = excluded.description,
                   korean_summary = excluded.korean_summary,
                   stars = excluded.stars,
                   forks = excluded.forks,
                   pull_requests = excluded.pull_requests,
                   pushes = excluded.pushes,
                   total_score = excluded.total_score,
                   contributor_logins = excluded.contributor_logins,
                   collection_names = excluded.collection_names"#,
            params![
                repo.date,
                repo.repo_id,
                repo.repo_name,
                repo.primary_language,
                repo.description,
                repo.korean_summary,
                repo.stars,
                repo.forks,
                repo.pull_requests,
                repo.pushes,
                repo.total_score,
                repo.contributor_logins,
                repo.collection_names,
            ],
        )?;

        Ok(())
    }

    pub fn save_repo_language(&self, lang: &RepoLanguage) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            r#"INSERT INTO repo_languages (date, repo_id, language, percentage)
               VALUES (?, ?, ?, ?)
               ON CONFLICT (date, repo_id, language) DO UPDATE SET
                   percentage = excluded.percentage"#,
            params![lang.date, lang.repo_id, lang.language, lang.percentage],
        )?;

        Ok(())
    }

    pub fn save_language_trend(&self, trend: &LanguageTrend) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            r#"INSERT INTO daily_language_trends (date, language, normalized_percentage, repo_count)
               VALUES (?, ?, ?, ?)
               ON CONFLICT (date, language) DO UPDATE SET
                   normalized_percentage = excluded.normalized_percentage,
                   repo_count = excluded.repo_count"#,
            params![trend.date, trend.language, trend.normalized_percentage, trend.repo_count],
        )?;

        Ok(())
    }

    pub fn get_trending_repos(&self, date: &str) -> Result<Vec<TrendingRepo>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"SELECT CAST(date AS VARCHAR), repo_id, repo_name, primary_language, description, korean_summary,
                      stars, forks, pull_requests, pushes, total_score, contributor_logins, collection_names
               FROM trending_repos WHERE date = ? ORDER BY total_score DESC"#
        )?;

        let repos = stmt.query_map(params![date], |row| {
            Ok(TrendingRepo {
                date: row.get(0)?,
                repo_id: row.get(1)?,
                repo_name: row.get(2)?,
                primary_language: row.get(3)?,
                description: row.get(4)?,
                korean_summary: row.get(5)?,
                stars: row.get(6)?,
                forks: row.get(7)?,
                pull_requests: row.get(8)?,
                pushes: row.get(9)?,
                total_score: row.get(10)?,
                contributor_logins: row.get(11)?,
                collection_names: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(repos)
    }

    pub fn get_repo_languages(&self, date: &str, repo_id: i64) -> Result<Vec<RepoLanguage>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"SELECT CAST(date AS VARCHAR), repo_id, language, percentage
               FROM repo_languages WHERE date = ? AND repo_id = ? ORDER BY percentage DESC"#
        )?;

        let langs = stmt.query_map(params![date, repo_id], |row| {
            Ok(RepoLanguage {
                date: row.get(0)?,
                repo_id: row.get(1)?,
                language: row.get(2)?,
                percentage: row.get(3)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(langs)
    }

    pub fn get_daily_language_trends(&self, date: &str) -> Result<Vec<LanguageTrend>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"SELECT CAST(date AS VARCHAR), language, normalized_percentage, repo_count
               FROM daily_language_trends WHERE date = ? ORDER BY normalized_percentage DESC"#
        )?;

        let trends = stmt.query_map(params![date], |row| {
            Ok(LanguageTrend {
                date: row.get(0)?,
                language: row.get(1)?,
                normalized_percentage: row.get(2)?,
                repo_count: row.get(3)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(trends)
    }

    pub fn get_weekly_language_trends(&self, end_date: &str) -> Result<Vec<LanguageTrend>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"SELECT ? as date, language, 
                      AVG(normalized_percentage) as normalized_percentage,
                      SUM(repo_count) as repo_count
               FROM daily_language_trends 
               WHERE date >= DATE(?, '-7 days') AND date <= ?
               GROUP BY language
               ORDER BY normalized_percentage DESC"#
        )?;

        let trends = stmt.query_map(params![end_date, end_date, end_date], |row| {
            Ok(LanguageTrend {
                date: row.get(0)?,
                language: row.get(1)?,
                normalized_percentage: row.get(2)?,
                repo_count: row.get(3)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(trends)
    }

    /// Check if a repo already exists for the given date with a Korean summary
    pub fn has_repo_with_summary(&self, date: &str, repo_id: i64) -> bool {
        let conn = self.conn.lock().unwrap();
        
        let result: Result<Option<i32>, _> = conn.query_row(
            r#"SELECT 1 FROM trending_repos 
               WHERE date = ? AND repo_id = ? AND korean_summary IS NOT NULL"#,
            params![date, repo_id],
            |row| row.get(0),
        );
        
        result.is_ok()
    }

    /// Get set of repo IDs that already have summaries for the given date
    pub fn get_existing_repo_ids(&self, date: &str) -> Result<std::collections::HashSet<i64>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"SELECT repo_id FROM trending_repos 
               WHERE date = ? AND korean_summary IS NOT NULL"#
        )?;

        let ids = stmt.query_map(params![date], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ids)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            conn: Arc::clone(&self.conn),
        }
    }
}
