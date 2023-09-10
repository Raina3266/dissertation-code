use std::{process::Command, sync::OnceLock};

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION},
    {Client, ClientBuilder},
};

static X_GOOG_USER_PROJECT: HeaderName = HeaderName::from_static("x-goog-user-project");

static CLIENTS: OnceLock<Clients> = OnceLock::new();

/// A struct that contains all the HTTP clients pre-configured with the requried authentication
/// data
///
/// Note, google trends doesn't need special config, since all auth is done through the 
pub struct Clients {
    pub bbc: Client,
    pub google_translate: Client,
    pub chatgpt: Client,
}

impl Clients {
    /// Get a reference to the global http client instance
    pub fn get() -> &'static Clients {
        CLIENTS.get_or_init(|| {
            let bbc = bbc_client();
            let google_translate = google_translate_client();
            let chatgpt = chatgpt_client();

            Clients {
                bbc,
                google_translate,
                chatgpt,
            }
        })
    }
}

fn bbc_client() -> Client {
    const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36";

    ClientBuilder::new().user_agent(USER_AGENT).build().unwrap()
}

/// Get an HTTP client authenticated for use with Google APIs
fn google_translate_client() -> Client {
    let secret = google_secret();
    let authorization = HeaderValue::from_str(&format!("Bearer {secret}")).unwrap();

    let project_id = std::env::var("GCLOUD_PROJECT_ID").unwrap();
    let project_id = HeaderValue::from_str(&project_id).unwrap();

    let headers = HeaderMap::from_iter([
        (AUTHORIZATION, authorization),
        (X_GOOG_USER_PROJECT.clone(), project_id),
    ]);

    ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap()
}

// Get the application secret from the `gcloud` CLI tool
fn google_secret() -> String {
    let output = Command::new("gcloud")
        .args(["auth", "print-access-token"])
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

/// Get an HTTP client authenticated for use with the ChatGPT client
fn chatgpt_client() -> Client {
    let openai_key = std::env::var("OPENAI_KEY").unwrap();
    let authorization = format!("Bearer {openai_key}").parse().unwrap();

    let headers = HeaderMap::from_iter([(AUTHORIZATION, authorization)]);

    ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap()
}
