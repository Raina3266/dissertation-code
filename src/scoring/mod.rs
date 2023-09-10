use crate::translate::Translations;
use color_eyre::{Report, Result};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokens::extract;

mod dedup;
mod tokens;
mod trends;

pub use trends::Trends;

use self::tokens::extract_chatgpt;

#[derive(Debug, Serialize)]
pub struct TranslationScores {
    pub google_uk_score: f64,
    pub google_us_score: f64,
    pub chatgpt_scores: HashMap<String, f64>,
}

/// Get the relative SEO optimization score for each translation
pub async fn score_translations(
    trends: &Trends,
    translations: &Translations,
    function_words: &HashSet<String>,
) -> Result<TranslationScores> {
    let mut google = extract(&translations.google, function_words);
    let mut chatgpt = extract_chatgpt(translations.chatgpt.clone(), function_words);

    let sets = core::iter::once(&mut google).chain(chatgpt.values_mut().map(|(set, _region)| set));

    // remove any words that are present in all sets
    dedup::dedup_sets(sets);

    let scores = TranslationScores {
        google_us_score: score_words(trends, &google, Region::America).await?,
        google_uk_score: score_words(trends, &google, Region::Britain).await?,
        chatgpt_scores: score_chatgpt(trends, chatgpt).await?,
    };

    Ok(scores)
}

async fn score_words(trends: &Trends, strings: &HashSet<String>, region: Region) -> Result<f64> {
    let scores = try_join_all(strings.iter().map(|s| trends.score(s, region))).await?;
    let sum: f64 = scores.iter().sum();
    let len = strings.len() as f64;

    Ok(sum / len)
}

async fn score_chatgpt(
    trends: &Trends,
    chatgpt: HashMap<String, (HashSet<String>, Region)>,
) -> Result<HashMap<String, f64>> {
    let futures = chatgpt
        .into_iter()
        .map(|(prompt_name, (strings, region))| async move {
            let score = score_words(trends, &strings, region).await?;
            Ok::<_, Report>((prompt_name, score))
        });

    let map = try_join_all(futures).await?.into_iter().collect();
    Ok(map)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Britain,
    America,
}

impl Region {
    pub fn to_str(&self) -> &'static str {
        match self {
            Region::Britain => "britain",
            Region::America => "america",
        }
    }

    #[allow(clippy::all)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "britain" => Some(Self::Britain),
            "america" => Some(Self::America),
            _ => None,
        }
    }
}
