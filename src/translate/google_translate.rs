use color_eyre::Report;
use serde::Deserialize;
use serde_json::json;

use crate::http_client::Clients;


const URL: &str = "https://translate.googleapis.com/v3beta1";

/// Translate the given string from Chinese to English using the Google Cloud Translate API
pub async fn google_translate(s: &str) -> Result<String, Report> {
    let project_id = std::env::var("GCLOUD_PROJECT_ID").unwrap();
    let url = format!("{URL}/projects/{project_id}:translateText");

    let client = &Clients::get().google_translate;
    let Response { translations } = client
        .post(url)
        .json(&json!({
            "contents": [ s ],
            "sourceLanguageCode": "zh-CN",
            "targetLanguageCode": "en-US",

        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(translations[0].translated_text.to_string())
}

#[derive(Deserialize)]
struct Response {
    translations: Vec<Translation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Translation {
    translated_text: String,
}
