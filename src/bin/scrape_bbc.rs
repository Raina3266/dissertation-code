use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use itertools::Itertools;
use tl::ParserOptions;

#[derive(Debug, Parser)]
pub struct Args {
    /// The path to the file containing the BBC topic URLs
    #[clap(long, short)]
    pub topics: PathBuf,

    /// The path to the output URL file
    #[clap(long, short)]
    pub output: PathBuf,
}

fn main() -> Result<()> {
    let Args { topics, output } = Args::parse();

    let mut output = BufWriter::new(File::create(output)?);

    let topics = load_urls(&topics)?;

    let urls: Vec<_> = topics
        .into_iter()
        .map(|(url, num_pages)| scrape_topic(&url, num_pages))
        .flatten_ok()
        .collect::<Result<_>>()?;

    let urls = dedup_urls(urls);

    for url in urls {
        writeln!(output, "{url}")?;
    }

    Ok(())
}

fn dedup_urls(urls: Vec<String>) -> Vec<String> {
    let mut result = Vec::with_capacity(urls.len());
    let mut seen = HashSet::new();

    for url in urls {
        if seen.contains(&url) {
            continue;
        } else {
            seen.insert(url.clone());
            result.push(url);
        }
    }

    result
}

fn load_urls(path: &Path) -> Result<Vec<(String, usize)>> {
    let text = std::fs::read_to_string(path)?;
    let vec = text
        .lines()
        .map(|s| {
            let mut words = s.split(' ');

            let url = words.next().unwrap();
            let num = words.next().unwrap().parse().unwrap();

            (url.to_string(), num)
        })
        .collect();

    Ok(vec)
}

fn scrape_topic(topic_url: &str, num_pages: usize) -> Result<Vec<String>> {
    (1..=num_pages)
        .map(|i| {
            let url = format!("{topic_url}?page={i}");
            let page = reqwest::blocking::get(url)?.text()?;
            articles(&page)
        })
        .flatten_ok()
        .collect()
}

/// Return all the article URLs on a page
fn articles(page: &str) -> Result<Vec<String>> {
    let html = tl::parse(page, ParserOptions::default())?;

    let articles = html
        .query_selector("a.bbc-uk8dsi")
        .ok_or_else(|| eyre!("no matching nodes"))?;

    let mut vec = Vec::with_capacity(20);

    for article in articles {
        let tag = article.get(html.parser()).unwrap().as_tag().unwrap();
        let link_bytes = tag.attributes().get("href").unwrap().unwrap();
        vec.push(link_bytes.as_utf8_str().to_string());
    }

    Ok(vec)
}
