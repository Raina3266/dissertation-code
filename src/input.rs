use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path, ops::Deref,
};

use color_eyre::Result;
use serde::Deserialize;

use crate::scoring::Region;

/// Read a file at a given path and return a Vec containing the individual lines
pub fn read_file_lines<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    BufReader::new(file).lines().collect()
}

/// A container for a set of prompts that we will give to chatgpt
///
/// This preserves the name of the prompt, so it can be used when writing the header for the CSV
/// file
pub struct ChatgptPrompts(HashMap<String, Prompt>);

impl ChatgptPrompts {
    /// Load prompts from a given json5 file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let map = json5::from_str(&s)?;
        Ok(Self(map))
    }
}

// allow access to the inner data
impl Deref for ChatgptPrompts {
    type Target = HashMap<String, Prompt>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize)]
pub struct Prompt {
    pub text: String,
    #[serde(default = "default_region")]
    pub region: Region,
}

fn default_region() -> Region {
    Region::America
}
