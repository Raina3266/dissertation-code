use std::io::Write;

use color_eyre::Result;
use csv::Writer;

use crate::{input::ChatgptPrompts, scoring::TranslationScores, translate::Translations};

#[derive(Default)]
pub struct CsvRow {
    pub url: String,
    pub translations: Option<Translations>,
    pub scores: Option<TranslationScores>,
}

impl CsvRow {
    pub fn new(
        url: String,
        translations: Option<Translations>,
        scores: Option<TranslationScores>,
    ) -> Self {
        Self {
            url,
            translations,
            scores,
        }
    }
}

/// Write every row to the given CSV output
pub fn write_csv(prompts: &ChatgptPrompts, out: impl Write, rows: &[CsvRow]) -> Result<()> {
    let mut writer = Writer::from_writer(out);

    write_header(&mut writer, prompts)?;

    for row in rows {
        write_row(&mut writer, row, prompts)?;
    }

    Ok(())
}

fn write_row(
    writer: &mut Writer<impl Write>,
    row: &CsvRow,
    prompts: &ChatgptPrompts,
) -> Result<()> {
    writer.write_field(&row.url)?;

    if let Some(translations) = &row.translations {
        let Translations {
            chinese_text,
            google,
            chatgpt,
        } = translations;

        writer.write_field(chinese_text)?;
        writer.write_field(google)?;

        for (string, _region) in chatgpt.values() {
            writer.write_field(string)?;
        }
    } else {
        let num_missing_fields = prompts.len() + 2;

        for _ in 0..num_missing_fields {
            writer.write_field("")?;
        }
    }

    if let Some(scores) = &row.scores {
        let TranslationScores {
            google_uk_score,
            google_us_score,
            chatgpt_scores,
        } = scores;

        let mut write_float = |float: f64| {
            if float.is_nan() {
                writer.write_field("")
            } else {
                writer.write_field(float.to_string())
            }
        };

        write_float(*google_us_score)?;
        write_float(*google_uk_score)?;

        for score in chatgpt_scores.values() {
            write_float(*score)?;
        }
    } else {
        let num_missing_fields = prompts.len() + 2;

        for _ in 0..num_missing_fields {
            writer.write_field("")?;
        }
    }

    // finish the row
    writer.write_record(core::iter::empty::<String>())?;

    Ok(())
}

fn write_header(writer: &mut Writer<impl Write>, prompts: &ChatgptPrompts) -> Result<()> {
    writer.write_field("url")?;
    writer.write_field("chinese_text")?;
    writer.write_field("google")?;

    for name in prompts.keys() {
        writer.write_field(name)?;
    }

    writer.write_field("google_us_score")?;
    writer.write_field("google_uk_score")?;

    for name in prompts.keys() {
        writer.write_field(format!("{name}_score"))?;
    }

    // finish the row
    writer.write_record(core::iter::empty::<String>())?;

    Ok(())
}
