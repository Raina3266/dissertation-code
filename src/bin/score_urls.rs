use clap::Parser;
use color_eyre::{eyre::Context, Help, Result};
use dissertation::{
    html,
    input::{self, ChatgptPrompts},
    output,
    scoring::{self, Trends},
    translate,
};
use futures::future::try_join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use output::CsvRow;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
pub struct Args {
    /// The path to the list of URLs
    #[clap(long, short)]
    pub urls: PathBuf,

    /// The path to the prompts file
    #[clap(long, short)]
    pub prompts: PathBuf,

    /// The path to the list of function words
    #[clap(long, short)]
    pub function_words: Option<PathBuf>,

    /// The path to the output CSV file
    #[clap(long, short)]
    pub output: PathBuf,

    /// Optional limit for the number of URLs to process
    #[clap(long, short)]
    pub limit: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()
        .context("no .env file found")
        .suggestion("create a file at `./.env` containing the required env vars")?;

    let Args {
        urls,
        prompts,
        function_words,
        output,
        limit,
    } = Args::parse();

    let urls = input::read_file_lines(urls)?;
    let function_words = match function_words {
        Some(path) => input::read_file_lines(path)?,
        None => vec![],
    };
    let function_words = function_words.into_iter().collect();
    let prompts = ChatgptPrompts::from_file(prompts)?;

    let out = make_output(&output)?;

    let progress = Progress::new(urls.len());
    let trends = Trends::new();

    let urls = urls
        .into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .map(|url| process_row(&progress, url, &function_words, &trends, &prompts));

    let rows = try_join_all(urls).await?;

    tracing::info!("writing output to {}", output.to_string_lossy());
    output::write_csv(&prompts, out, &rows)?;

    Ok(())
}

async fn process_row(
    progress: &Progress,
    url: String,
    function_words: &HashSet<String>,
    trends: &Trends,
    prompts: &ChatgptPrompts,
) -> Result<CsvRow> {
    let Ok(chinese_description)= html::description_of_page(&url).await else {
        return Ok(CsvRow::new(url, None, None));
    };
    progress.descriptions.inc(1);

    let Ok(translations) = translate::translate(&chinese_description, prompts).await else {
        return Ok(CsvRow::new(url, None, None));
    };
    progress.translations.inc(1);

    let Ok(scores) = scoring::score_translations(trends, &translations, function_words).await else {
        return Ok(CsvRow::new(url, Some(translations), None));
    };
    progress.scores.inc(1);

    let row = CsvRow::new(url, Some(translations), Some(scores));

    Ok(row)
}

fn make_output(path: &Path) -> Result<impl Write, std::io::Error> {
    let file = File::create(path)?;
    Ok(BufWriter::new(file))
}

struct Progress {
    _multi: MultiProgress,
    descriptions: ProgressBar,
    translations: ProgressBar,
    scores: ProgressBar,
}

impl Progress {
    fn new(count: usize) -> Self {
        let sty = ProgressStyle::with_template("{msg:26}: {bar} {pos:>7}/{len:7} ").unwrap();

        let count = count as u64;
        let multi = MultiProgress::new();

        let descriptions = multi.insert(
            0,
            ProgressBar::new(count)
                .with_style(sty.clone())
                .with_message("Getting descriptions"),
        );

        let translations = multi.insert(
            1,
            ProgressBar::new(count)
                .with_style(sty.clone())
                .with_message("Translating descriptions"),
        );

        let scores = multi.insert(
            2,
            ProgressBar::new(count)
                .with_style(sty)
                .with_message("Scoring translations"),
        );

        multi.set_move_cursor(true);

        Self {
            _multi: multi,
            descriptions,
            translations,
            scores,
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.descriptions.finish();
        self.translations.finish();
        self.scores.finish();
    }
}
