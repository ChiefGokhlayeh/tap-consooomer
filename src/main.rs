extern crate pest;
#[macro_use]
extern crate pest_derive;

use clap::Parser as ClapParser;
use pest::Parser;
use std::fs;

#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser, value_name = "FILE")]
    tap_file: String,
}

#[derive(Parser)]
#[grammar = "tap.pest"]
pub struct TAPParser;

fn main() {
    let cli = Cli::parse();

    let contents = fs::read_to_string(cli.tap_file).expect("Failed to read file");
    let tap_document = TAPParser::parse(Rule::document, &contents)
        .into_iter()
        .next()
        .expect("Failed to parse TAP document");

    println!("{:}", tap_document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common() {
        let contents = fs::read_to_string("examples/common.log").expect("Failed to read file");

        let tap_pairs = TAPParser::parse(Rule::document, &contents);
        assert!(tap_pairs.is_ok());
        let tap_document = tap_pairs.into_iter().next();
        assert!(tap_document.is_some());

        println!("{:?}", Some(tap_document))
    }
}
