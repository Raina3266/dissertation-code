use color_eyre::{eyre::bail, Report, Result};
use moka::future::Cache;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::named_params;
use serde::Deserialize;

use crate::rate_limiter::RateLimiters;

use super::Region;

/// A struct that manages Google Trends data
///
/// It stores data in the database, and also in a high-speed, async-aware, in-memory cache
pub struct Trends {
    pool: Pool<SqliteConnectionManager>,
    cache: Cache<(String, Region), f64>,
}

impl Default for Trends {
    fn default() -> Self {
        Self::new()
    }
}

impl Trends {
    pub fn new() -> Self {
        let pool = Self::open_db();
        let rows = Self::all_scores(&pool).unwrap();
        let cache = Cache::new(10_000_000);

        for row in rows {
            cache.blocking().insert((row.text, row.region), row.score);
        }

        Self { pool, cache }
    }

    fn open_db() -> Pool<SqliteConnectionManager> {
        let pool = Pool::new(SqliteConnectionManager::file("./trends.db")).unwrap();
        let sql = include_str!("./create_table.sql");
        pool.get().unwrap().execute(sql, []).unwrap();

        pool
    }

    pub async fn score(&self, word: &str, region: Region) -> Result<f64> {
        let entry = self
            .cache
            .entry((word.to_string(), region))
            .or_try_insert_with(self.get_score_and_cache(word, region))
            .await
            .map_err(|e| color_eyre::eyre::eyre!("{e}"))?;

        Ok(*entry.value())
    }

    async fn get_score_and_cache(&self, word: &str, region: Region) -> Result<f64> {
        match self.load_score_from_db(word, region).await? {
            Some(score) => Ok(score),
            None => {
                let score = fetch_score_from_trends(word, region).await?;
                self.store_score_to_db(word, region, score).await?;
                Ok(score)
            }
        }
    }

    async fn load_score_from_db(&self, word: &str, region: Region) -> Result<Option<f64>> {
        let conn = self.pool.get()?;
        let word = word.to_string();

        tokio::task::spawn_blocking(move || {
            let sql = "SELECT * FROM trends WHERE text = (?1) AND region = (?2)";
            let mut statement = conn.prepare_cached(sql).unwrap();
            let mut rows = statement.query([word, region.to_str().to_string()])?;
            let score = rows.next()?.map(|row| row.get("score")).transpose()?;

            Ok(score)
        })
        .await
        .unwrap()
    }

    async fn store_score_to_db(&self, word: &str, region: Region, score: f64) -> Result<()> {
        let conn = self.pool.get()?;
        let word = word.to_string();

        tokio::task::spawn_blocking(move || -> Result<_> {
            let sql = "INSERT INTO trends (text, region, score) VALUES (:word, :region, :score)";
            let mut statement = conn.prepare_cached(sql).unwrap();
            statement.execute(named_params! {
                ":word": word,
                ":region": region.to_str(),
                ":score": score,
            })?;

            Ok(())
        })
        .await??;

        Ok(())
    }

    fn all_scores(pool: &Pool<SqliteConnectionManager>) -> Result<Vec<Row>> {
        let conn = pool.get()?;

        let mut statement = conn.prepare("SELECT * FROM trends")?;

        let rows = statement.query_map([], |row| {
            Ok(Row {
                text: row.get("text")?,
                region: Region::from_str(&row.get::<_, String>("region")?).unwrap(),
                score: row.get("score")?,
            })
        })?;

        rows.into_iter()
            .map(|res| res.map_err(Report::from))
            .collect()
    }
}

struct Row {
    text: String,
    region: Region,
    score: f64,
}

/// Get the relative popularity score for a keyword from the Google Trends API
async fn fetch_score_from_trends(word: &str, region: Region) -> Result<f64> {
    let secret = std::env::var("GCLOUD_KEY").unwrap();
    let region = match region {
        Region::Britain => "GB",
        Region::America => "US",
    };
    let url = format!("https://www.googleapis.com/trends/v1beta/graph?terms={word}&key={secret}&restrictions_geo={region}");

    RateLimiters::get().wait_trends().await;

    let lines = match reqwest::get(&url).await?.json().await? {
        Response::Ok { lines } => lines,
        Response::Err(e) => bail!("{e}"),
    };

    let value = lines[0].points.last().unwrap().value;

    Ok(value)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    Ok { lines: Vec<Line> },
    Err(serde_json::Value),
}

#[derive(Deserialize)]
struct Line {
    points: Vec<Point>,
}

#[derive(Deserialize)]
struct Point {
    value: f64,
}

#[cfg(test)]
mod tests {
    use futures::future::try_join_all;

    use super::*;

    #[tokio::test]
    async fn can_google_trends_concurrent() {
        dotenvy::dotenv().unwrap();

        let trends = Trends::new();

        let strings: Vec<_> = ('a'..='z')
            .flat_map(|c1| ('a'..='z').map(move |c2| format!("{c1}{c2}")))
            .collect();

        try_join_all(strings.iter().map(|s| trends.score(s, Region::Britain)))
            .await
            .unwrap();
    }
}
