use color_eyre::Report;
use serde::Deserialize;
use serde_json::json;

use crate::{rate_limiter::RateLimiters, http_client::Clients};

/// Ask chatgpt the given prompt
///
/// This behaves similarly to:
///  - opening a brand new chat conversation with chatgpt
///  - asking the given prompt
///  (i.e. each message is treated as a fresh conversation)
pub async fn ask_chatgpt(prompt: &str) -> Result<String, Report> {
    let client = &Clients::get().chatgpt;
    RateLimiters::get().wait_chatgpt().await;

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            { "role": "user", "content": prompt },
        ],
        "temperature": 0,
    });

    let choices = loop {
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        match response {
            Response::Ok { choices } => break choices,
            Response::Err { code, content } => match code {
                // if there is no error code, or is it 5XX, retry
                None | Some(500..=599) => continue,
                Some(x) => panic!("code {x}, error: {content}"),
            },
        };
    };

    let result = choices
        .into_iter()
        .map(|c| c.message.content)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(result.trim_matches('"').to_string())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    Ok {
        choices: Vec<Choice>,
    },
    Err {
        code: Option<u32>,
        #[serde(flatten)]
        content: serde_json::Value,
    },
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}
