use clap::Parser as ClapParser;
use std::{fs, io::Read};

#[derive(ClapParser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = concat!("Reads a given Test Anything Protocol (TAP) file ",
    "and prints the JSON-formatted parser result to stdout. If FILE is ",
    "omitted, TAP input is read from stdin. Parsing only comences after ",
    "encountering an EOF. Only complete TAP files are supported.")
)]
struct Cli {
    /// Path to TAP input file.
    #[clap(value_parser, value_name = "FILE")]
    tap_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let content = cli.tap_file.map_or_else(
        || {
            let mut buf = String::with_capacity(4096);
            std::io::stdin()
                .read_to_string(&mut buf)
                .map(move |_| buf)
                .unwrap_or_else(|_| panic!("Failed to read from stdin"))
        },
        |file| {
            fs::read_to_string(&file).unwrap_or_else(|_| panic!("Failed to read file, {}", &file))
        },
    );
    let document = tap::Document::parse_from_str(&content).expect("Failed to parse TAP document");
    println!(
        "{}",
        serde_json::to_string_pretty(&document).expect("Failed to serialize TAP document")
    )
}
