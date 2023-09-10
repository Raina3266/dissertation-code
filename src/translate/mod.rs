mod chatgpt;
mod google_translate;

use std::collections::HashMap;

use color_eyre::Result;

use crate::{
    input::{ChatgptPrompts, Prompt},
    scoring::Region,
};

/// Generate all translations of the input text
pub async fn translate(chinese_text: &str, prompts: &ChatgptPrompts) -> Result<Translations> {
    let google = google_translate::google_translate(chinese_text).await?;

    let mut chatgpt = HashMap::new();

    for (name, Prompt { text, region }) in prompts.iter() {
        let prompt = text.replace("{chinese}", chinese_text);
        let translated = chatgpt::ask_chatgpt(&prompt).await?;
        chatgpt.insert(name.to_string(), (translated, *region));
    }

    Ok(Translations {
        chinese_text: chinese_text.into(),
        google,
        chatgpt,
    })
}

#[derive(Debug)]
pub struct Translations {
    pub chinese_text: String,
    pub google: String,
    pub chatgpt: HashMap<String, (String, Region)>,
}
