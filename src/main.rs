extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "tap.pest"]
pub struct TAPParser;

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;
    use std::fs;

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
