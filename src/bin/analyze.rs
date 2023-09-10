use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::Result;
use polars::{lazy::dsl::col, prelude::*};
use statrs::distribution::{ContinuousCDF, Normal};

#[derive(Debug, Parser)]
struct Args {
    /// The input CSV file
    #[clap(long, short)]
    input: PathBuf,
}

fn main() -> Result<()> {
    let Args { input } = Args::parse();

    let df = CsvReader::from_path(&input)?
        .infer_schema(None)
        .has_header(true)
        .finish()?;

    let columns = [
        "google_us_score",
        "google_uk_score",
        "default_english_us_score",
        "default_english_uk_score",
        "american_english_us_score",
        "british_english_uk_score",
        "american_english_seo_us_score",
        "british_english_seo_uk_score",
    ];

    let scores = df
        .lazy()
        .drop_nulls(None)
        .select(columns.map(col))
        .collect()?
        .drop_nulls::<String>(None)?;

    let num_rows = scores.height();
    let mut mean_and_std = scores.mean().vstack(&scores.std(1)).unwrap();

    let mut file = File::create("analysis.csv").unwrap();
    CsvWriter::new(&mut file).finish(&mut mean_and_std).unwrap();

    println!("{mean_and_std}");
    println!("rows: {num_rows}");

    for col1 in columns {
        for col2 in columns {
            if col1 == col2 {
                continue;
            }

            if col1.contains("google") || col2.contains("google") {
                continue;
            }

            let z = z_score(mean_and_std.clone(), col1, col2, num_rows);
            if z.abs() > 2.0 {
                println!("{col1} <=> {col2} - {z}");
            }
        }
    }

    let z_score_product: Vec<Vec<_>> = columns
        .iter()
        .map(|col1| {
            columns
                .iter()
                .map(|col2| z_score(mean_and_std.clone(), col1, col2, num_rows))
                .collect()
        })
        .collect();

    let file = File::create("z_scores.csv").unwrap();
    let file = BufWriter::new(file);

    write_table(file, z_score_product, &columns)?;

    let p_value_product: Vec<Vec<_>> = columns
        .iter()
        .map(|col1| {
            columns
                .iter()
                .map(|col2| {
                    let z_score = z_score(mean_and_std.clone(), col1, col2, num_rows);
                    p_value_from_z_score(z_score)
                })
                .collect()
        })
        .collect();

    let file = File::create("p_values.csv").unwrap();
    let file = BufWriter::new(file);

    write_table(file, p_value_product, &columns)?;

    Ok(())
}

/// Convert a z-score to a p-value by using the cumulative distribution function for the standard
/// normal distribution (the normal distribution with mean 0 and S.D. 1)
fn p_value_from_z_score(z_score: f64) -> f64 {
    let z = -1.0 * z_score.abs();
    Normal::new(0.0, 1.0).unwrap().cdf(z)
}

fn z_score(df: DataFrame, col1: &str, col2: &str, n: usize) -> f64 {
    let n = n as f64;
    let (mean_1, std_1) = mean_and_std(df.clone(), col1);
    let (mean_2, std_2) = mean_and_std(df, col2);

    (mean_2 - mean_1) / ((std_2.powi(2) / n) + std_1.powi(2) / n).sqrt()
}

fn mean_and_std(df: DataFrame, col: &str) -> (f64, f64) {
    let vec = df.column(col).unwrap().f64().unwrap().to_vec();
    (vec[0].unwrap(), vec[1].unwrap())
}

fn write_table(out: impl Write, data: Vec<Vec<f64>>, column_names: &[&'static str]) -> Result<()> {
    let mut writer = csv::Writer::from_writer(out);

    // write header
    writer.write_field("")?; // empty cell
    for name in column_names {
        writer.write_field(name)?;
    }
    writer.write_record(None::<&[u8]>)?; // new line

    for (i, row) in data.iter().enumerate() {
        writer.write_field(column_names[i])?;

        for cell in row {
            writer.write_field(format!("{cell}"))?;
        }

        writer.write_record(None::<&[u8]>)?; // new line
    }

    Ok(())
}
