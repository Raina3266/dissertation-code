use color_eyre::{eyre::bail, Result};
use tl::ParserOptions;

use crate::{rate_limiter::RateLimiters, http_client::Clients};

/// Load the given URL, parse the HTML, and return the content of the meta description tag (if it
/// exists)
pub async fn description_of_page(url: &str) -> Result<String> {
    RateLimiters::get().wait_bbc().await;

    let html = load_html(url).await?;
    let Some(description) = extract_desc_from_string(&html)? else {
        bail!("page had no description tag");
    };

    Ok(description)
}

async fn load_html(url: &str) -> Result<String> {
    let client = &Clients::get().bbc;
    let response = client.get(url).send().await?;
    Ok(response.text().await?)
}

fn extract_desc_from_string(html: &str) -> Result<Option<String>> {
    let parsed = tl::parse(html, ParserOptions::default())?;
    Ok(extract_html_from_vdom(parsed))
}

fn extract_html_from_vdom(dom: tl::VDom) -> Option<String> {
    // find the first meta tag that has the attribute `name="description"`
    let node = dom
        .query_selector(r#"meta[name="description"]"#)?
        .next()?
        .get(dom.parser())?;

    // Get the value of the `content` attribute
    let content = node.as_tag()?.attributes().get("content")??;

    Some(content.as_utf8_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let html = r#"<head><meta name="description" content="foo"><\head>"#;
        let parsed = extract_desc_from_string(html).unwrap().unwrap();
        assert_eq!(parsed, "foo");
    }

    #[tokio::test]
    async fn bbc_article_has_desc() {
        description_of_page("https://www.bbc.co.uk/news/world-asia-66414696")
            .await
            .unwrap();
    }
}
