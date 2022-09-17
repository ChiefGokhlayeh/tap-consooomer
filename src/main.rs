extern crate pest;
#[macro_use]
extern crate pest_derive;

use anyhow::{anyhow, Result};
use clap::Parser as ClapParser;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use serde::Serialize;
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

#[derive(Parser)]
#[grammar = "tap14.pest"]
struct TAPParser;

#[derive(Debug, Serialize)]
struct Preamble<'a> {
    version: &'a str,
}

#[derive(Debug, Serialize)]
struct Plan<'a> {
    first: i32,
    last: i32,
    reason: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct Body<'a> {
    statements: Vec<Statement<'a>>,
}

#[derive(Debug, Serialize)]
struct Pragma<'a> {
    key: &'a str,
}

#[derive(Debug, Serialize)]
struct BailOut<'a> {
    reason: Option<&'a str>,
}

#[derive(Debug, Serialize)]
enum Key {
    Skip,
    Todo,
}

#[derive(Debug, Serialize)]
struct Directive<'a> {
    key: Key,
    reason: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct Test<'a> {
    result: bool,
    number: Option<i32>,
    description: Option<&'a str>,
    directive: Option<Directive<'a>>,
    yaml: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
struct Subtest<'a> {
    name: Option<&'a str>,
    plan: Plan<'a>,
    body: Vec<Statement<'a>>,
}

#[derive(Debug, Serialize)]
enum Statement<'a> {
    Anything(&'a str),
    BailOut(BailOut<'a>),
    Pragma(Pragma<'a>),
    Subtest(Subtest<'a>),
    Test(Test<'a>),
}

#[derive(Debug, Serialize)]
struct Document<'a> {
    preamble: Preamble<'a>,
    plan: Plan<'a>,
    body: Vec<Statement<'a>>,
}

#[derive(Debug)]
enum DocumentContent<'a> {
    Plan(Plan<'a>),
    Body(Vec<Statement<'a>>),
}

#[derive(Debug)]
enum SubtestContent<'a> {
    Plan(Plan<'a>),
    Statement(Statement<'a>),
}

fn parse_preamble(mut pairs: Pairs<Rule>) -> Preamble {
    Preamble {
        version: pairs.next().unwrap().as_str(),
    }
}

fn parse_plan(mut pairs: Pairs<'_, Rule>) -> Result<Plan<'_>> {
    Ok(Plan {
        first: pairs.next().unwrap().as_str().parse()?,
        last: pairs.next().unwrap().as_str().parse()?,
        reason: pairs.next().map(|r| r.as_str()),
    })
}

fn parse_directive(mut pairs: Pairs<'_, Rule>) -> Result<Directive> {
    let key = pairs.next().unwrap().as_str().to_lowercase();
    Ok(Directive {
        key: match key.as_str() {
            "skip" => Ok(Key::Skip),
            "todo" => Ok(Key::Todo),
            _ => Err(anyhow!("Directive key '{}' must be 'skip' or 'todo'", key)),
        }?,
        reason: pairs.next().map(|p| p.as_str()),
    })
}

fn parse_yaml_block(pairs: Pairs<'_, Rule>) -> Vec<&'_ str> {
    pairs.map(|p| p.as_str()).collect()
}

fn parse_test(mut pairs: Pairs<'_, Rule>) -> Result<Test<'_>> {
    let pair = pairs.next().unwrap();
    let result = match pair.as_str().to_lowercase().as_str() {
        "ok" => Ok(true),
        "not ok" => Ok(false),
        _ => Err(anyhow!(
            "Result '{}' must be 'ok' or 'not ok'",
            pair.as_str()
        )),
    }?;
    let mut number: Option<i32> = None;
    let mut description = None;
    let mut directive = None;
    let mut yaml = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::number => number = pair.as_str().parse::<i32>().ok(),
            Rule::description => description = Some(pair.as_str()),
            Rule::directive => directive = parse_directive(pair.into_inner()).ok(),
            Rule::yaml_block => yaml.append(&mut parse_yaml_block(pair.into_inner())),
            _ => unreachable!(),
        };
    }
    Ok(Test {
        result,
        number,
        description,
        directive,
        yaml,
    })
}

fn parse_bail_out(mut pairs: Pairs<'_, Rule>) -> Result<BailOut> {
    Ok(BailOut {
        reason: pairs.next().map(|p| p.as_str()),
    })
}

fn parse_pragma(mut pairs: Pairs<'_, Rule>) -> Result<Pragma> {
    Ok(Pragma {
        key: pairs.next().unwrap().as_str(),
    })
}

fn parse_subtest(mut pairs: Pairs<'_, Rule>) -> Result<Subtest> {
    let pair = pairs.next().unwrap();
    let name = match pair.as_rule() {
        Rule::name => Some(pair.as_str()),
        _ => None,
    };

    let mut contents = vec![]; /* FIXME: There has to be a better way to split plan and body! */

    /* Consume first pair if not consumed by 'name'. */
    if name.is_none() {
        contents.push(match pair.as_rule() {
            Rule::plan => parse_plan(pair.into_inner()).map(SubtestContent::Plan),
            _ => parse_statement(pair).map(SubtestContent::Statement),
        }?)
    }

    /* Now consume the rest of the pairs. */
    let mut rest: Vec<SubtestContent> = pairs
        .map(|p| match p.as_rule() {
            Rule::plan => parse_plan(p.into_inner()).map(SubtestContent::Plan),
            _ => parse_statement(p).map(SubtestContent::Statement),
        })
        .collect::<Result<Vec<SubtestContent>>>()?;
    contents.append(&mut rest);

    let mut p = None;
    let mut i = 0;
    while i < contents.len() {
        /* TODO: Use drain_filter once stable. */
        if matches!(contents[i], SubtestContent::Plan(_)) {
            p = Some(match contents.remove(i) {
                SubtestContent::Plan(p) => p,
                SubtestContent::Statement(_) => unreachable!(),
            })
        }
        i += 1;
    }
    let plan = p.unwrap();
    let body = contents
        .into_iter()
        .filter_map(|s| match s {
            SubtestContent::Plan(_) => None,
            SubtestContent::Statement(s) => Some(s),
        })
        .collect();

    Ok(Subtest { name, plan, body })
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement> {
    match pair.as_rule() {
        Rule::test => Ok(Statement::Test(parse_test(pair.into_inner())?)),
        Rule::bail_out => Ok(Statement::BailOut(parse_bail_out(pair.into_inner())?)),
        Rule::pragma => Ok(Statement::Pragma(parse_pragma(pair.into_inner())?)),
        Rule::subtest => Ok(Statement::Subtest(parse_subtest(pair.into_inner())?)),
        Rule::anything => Ok(Statement::Anything(pair.as_str())),
        _ => unreachable!(),
    }
}

fn parse_document_content(content: Pair<Rule>) -> Result<DocumentContent> {
    Ok(match content.as_rule() {
        Rule::plan => DocumentContent::Plan(parse_plan(content.into_inner())?),
        Rule::body => {
            let statements: Result<Vec<_>> = content.into_inner().map(parse_statement).collect();
            DocumentContent::Body(statements?)
        }
        _ => unreachable!(),
    })
}

fn split_contents<'a>(
    content1: DocumentContent<'a>,
    content2: DocumentContent<'a>,
) -> (Plan<'a>, Vec<Statement<'a>>) {
    let (plan, body) = match content1 {
        DocumentContent::Plan(p) => (
            p,
            match content2 {
                DocumentContent::Body(b) => b,
                _ => panic!("Unexpected double 'body'"),
            },
        ),
        DocumentContent::Body(b) => (
            match content2 {
                DocumentContent::Plan(p) => p,
                _ => panic!("Unexpected double 'plan'"),
            },
            b,
        ),
    };
    (plan, body)
}

fn parse_document(pair: Pair<Rule>) -> Result<Document> {
    match pair.as_rule() {
        Rule::document => {
            let mut pairs = pair.into_inner();
            let preamble = parse_preamble(pairs.next().unwrap().into_inner());

            let content1 = parse_document_content(pairs.next().unwrap())?;
            let content2 = parse_document_content(pairs.next().unwrap())?;
            let (plan, body) = split_contents(content1, content2);

            Ok(Document {
                preamble,
                plan,
                body,
            })
        }
        _ => Err(anyhow!("Unexpected '{}'", pair.as_str())),
    }
}

fn parse_document_from_str(content: &str) -> Result<Option<Document>> {
    if let Some(pair) = TAPParser::parse(Rule::document, content)?.next() {
        parse_document(pair).map(Some)
    } else {
        Ok(None)
    }
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
    if let Some(document) = parse_document_from_str(&content).expect("Failed to parse TAP document")
    {
        println!(
            "{}",
            serde_json::to_string_pretty(&document).expect("Failed to serialize TAP document")
        )
    }
}

#[cfg(test)]
mod tests {
    use pest::consumes_to;
    use pest::parses_to;

    use super::*;

    #[test]
    fn test_version() {
        parses_to! {
            parser: TAPParser,
            input : "TAP version 14",
            rule: Rule::preamble,
            tokens: [
                preamble(0, 14, [
                    version(12, 14)
                ])
            ]
        }
    }

    #[test]
    fn test_version_mixed_case() {
        parses_to! {
            parser: TAPParser,
            input : "tAp VeRsIoN 123",
            rule: Rule::preamble,
            tokens: [
                preamble(0, 15, [
                    version(12, 15)
                ])
            ]
        }
    }

    #[test]
    fn test_plan() {
        parses_to! {
            parser: TAPParser,
            input : "1..2",
            rule: Rule::plan,
            tokens: [
                plan(0, 4, [
                    first(0, 1), last(3, 4)
                ])
            ]
        }
    }

    #[test]
    fn test_plan_with_reason() {
        parses_to! {
            parser: TAPParser,
            input : "1..20 # generated",
            rule: Rule::plan,
            tokens: [
                plan(0, 17, [
                    first(0, 1), last(3, 5), reason(8, 17)
                ])
            ]
        }
    }

    #[test]
    fn test_ok_plain() {
        parses_to! {
            parser: TAPParser,
            input : "ok",
            rule: Rule::test,
            tokens: [
                test(0, 2, [
                    result(0, 2)
                ])
            ]
        }
    }

    #[test]
    fn test_not_ok_plain() {
        parses_to! {
            parser: TAPParser,
            input : "not ok",
            rule: Rule::test,
            tokens: [
                test(0, 6, [
                    result(0, 6)
                ])
            ]
        }
    }

    #[test]
    fn test_not_ok_plain_mixed_case() {
        parses_to! {
            parser: TAPParser,
            input : "nOt Ok",
            rule: Rule::test,
            tokens: [
                test(0, 6, [
                    result(0, 6)
                ])
            ]
        }
    }

    #[test]
    fn test_ok_with_number() {
        parses_to! {
            parser: TAPParser,
            input : "ok 123",
            rule: Rule::test,
            tokens: [
                test(0, 6, [
                    result(0, 2), number(3, 6)
                ])
            ]
        }
    }

    #[test]
    fn test_ok_with_description() {
        parses_to! {
            parser: TAPParser,
            input : "ok - hello world",
            rule: Rule::test,
            tokens: [
                test(0, 16, [
                    result(0, 2), description(5, 16)
                ])
            ]
        }
    }

    #[test]
    fn test_not_ok_with_description() {
        parses_to! {
            parser: TAPParser,
            input : "not ok - hello world",
            rule: Rule::test,
            tokens: [
                test(0, 20, [
                    result(0, 6), description(9, 20)
                ])
            ]
        }
    }

    #[test]
    fn test_ok_with_description_no_dash() {
        parses_to! {
            parser: TAPParser,
            input : "ok hello world",
            rule: Rule::test,
            tokens: [
                test(0, 14, [
                    result(0, 2), description(3, 14)
                ])
            ]
        }
    }

    #[test]
    fn test_ok_with_directive_skip() {
        parses_to! {
            parser: TAPParser,
            input : "ok # skip",
            rule: Rule::test,
            tokens: [
                test(0, 9, [
                    result(0, 2), directive(3, 9, [
                        key(5, 9)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_not_ok_with_directive_todo_reason() {
        parses_to! {
            parser: TAPParser,
            input : "not ok # todo this is a reason",
            rule: Rule::test,
            tokens: [
                test(0, 30, [
                    result(0, 6), directive(7, 30, [
                        key(9, 13), reason(14, 30)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_not_ok_with_number_directive_skip_reason() {
        parses_to! {
            parser: TAPParser,
            input : "not ok 42 # todo this is a reason",
            rule: Rule::test,
            tokens: [
                test(0, 33, [
                    result(0, 6), number(7, 9), directive(10, 33, [
                        key(12, 16), reason(17, 33)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_ok_with_number_description_directive_skip_reason() {
        parses_to! {
            parser: TAPParser,
            input : "ok 1 - hello world # skip this is a reason",
            rule: Rule::test,
            tokens: [
                test(0, 42, [
                    result(0, 2), number(3, 4), description(7, 19), directive(19, 42, [
                        key(21, 25),
                        reason(26, 42)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_bail_out() {
        parses_to! {
            parser: TAPParser,
            input : "bail out!",
            rule: Rule::bail_out,
            tokens: [
                bail_out(0, 9)
            ]
        }
    }

    #[test]
    fn test_bail_out_mixed_case() {
        parses_to! {
            parser: TAPParser,
            input : "BaIl OuT!",
            rule: Rule::bail_out,
            tokens: [
                bail_out(0, 9)
            ]
        }
    }

    #[test]
    fn test_bail_out_with_reason() {
        parses_to! {
            parser: TAPParser,
            input : "bail out! something went terribly wrong",
            rule: Rule::bail_out,
            tokens: [
                bail_out(0, 39, [
                    reason(10, 39)
                ])
            ]
        }
    }

    #[test]
    fn test_pragma() {
        parses_to! {
            parser: TAPParser,
            input : "pragma -strict",
            rule: Rule::pragma,
            tokens: [
                pragma(0, 14, [
                    pragma_key(8, 14)
                ])
            ]
        }
    }

    #[test]
    fn test_pragma_mixed_case() {
        parses_to! {
            parser: TAPParser,
            input : "pRaGmA +strict",
            rule: Rule::pragma,
            tokens: [
                pragma(0, 14, [
                    pragma_key(8, 14)
                ])
            ]
        }
    }

    #[test]
    fn test_subtest_with_empty_declaration() {
        parses_to! {
            parser: TAPParser,
            input : r#"# subtest
        1..20"#,
            rule: Rule::subtest,
            tokens: [
                subtest(0, 23, [
                    plan(18, 23, [
                        first(18, 19), last(21, 23)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_subtest_with_declaration() {
        parses_to! {
            parser: TAPParser,
            input : concat!(
                "# Subtest: This is a subtest\n",
                "    1..20"
            ),
            rule: Rule::subtest,
            tokens: [
                subtest(0, 38, [
                    name(11, 28),
                    plan(33, 38, [
                        first(33, 34), last(36, 38)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_subtest_without_declaration() {
        parses_to! {
            parser: TAPParser,
            input : "    1..20",
            rule: Rule::subtest,
            tokens: [
                subtest(0, 9, [
                    plan(4, 9, [
                        first(4, 5), last(7, 9)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_subtest_with_plan_body() {
        parses_to! {
            parser: TAPParser,
            input : concat!(
                "    1..21\n",
                "    ok 1 - hello world\n",
            ),
            rule: Rule::subtest,
            tokens: [
                subtest(0, 33, [
                    plan(4, 9, [
                        first(4, 5), last(7, 9)
                    ]),
                    test(14, 32, [
                        result(14, 16), number(17, 18), description(21, 32)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_subtest_with_body_plan() {
        parses_to! {
            parser: TAPParser,
            input : concat!(
                "    ok 1 - hello world\n",
                "    1..21\n",
            ),
            rule: Rule::subtest,
            tokens: [
                subtest(0, 32, [
                    test(4, 22, [
                        result(4, 6), number(7, 8), description(11, 22)
                    ]),
                    plan(27, 32, [
                        first(27, 28), last(30, 32)
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_comment() {
        parses_to! {
            parser: TAPParser,
            input : "# this is a comment",
            rule: Rule::COMMENT,
            tokens: [
            ]
        }
    }

    #[test]
    fn test_yaml_block() {
        parses_to! {
            parser: TAPParser,
            input : concat!(
                "  ---\n",
                "  YAML line 1\n",
                "  YAML line 2\n",
                "  YAML line 3\n",
                "  ok 1 - this is considered YAML\n",
                "  0..1 # even this is YAML\n",
                "  ...\n",
            ),
            rule: Rule::yaml_block,
            tokens: [
                yaml_block(0, 113, [
                    yaml(8, 19),
                    yaml(22, 33),
                    yaml(36, 47),
                    yaml(50, 80),
                    yaml(83, 107),
                ])
            ]
        }
    }

    #[test]
    fn test_common() {
        let contents = fs::read_to_string("examples/common.tap").expect("Failed to read file");

        parses_to! {
            parser: TAPParser,
            input : &contents,
            rule: Rule::document,
            tokens: [
                document(0, 318, [
                    preamble(0, 14, [
                        version(12, 14)
                    ]),
                    plan(15, 19, [
                        first(15, 16), last(18, 19)
                    ]),
                    body(19, 318, [
                        test(93, 131, [
                            result(93, 95),
                            number(96, 97),
                            description(100, 122),
                            directive(122, 131, [
                                key(123, 127), reason(128, 131)
                            ])
                        ]),
                        test(132, 171, [
                            result(132, 134),
                            number(135, 136),
                            description(139, 158),
                            directive(158, 171, [
                                key(160, 164), reason(165, 171)
                            ])
                        ]),
                        test(172, 209, [
                            result(172, 174), number(175, 176), description(179, 200)
                        ]),
                        test(210, 252, [
                            result(210, 212), number(213, 214), description(217, 252)
                        ]),
                        test(253, 294, [
                            result(253, 255), number(256, 257), description(260, 294)
                        ]),
                        test(295, 317, [
                            result(295, 297), number(298, 299), description(302, 317)
                        ]),
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_cascading() {
        let contents = fs::read_to_string("examples/cascading.tap").expect("Failed to read file");

        parses_to! {
            parser: TAPParser,
            input : &contents,
            rule: Rule::document,
            tokens: [
                document(0, 300, [
                    preamble(0, 14, [
                        version(12, 14)
                    ]),
                    plan(15, 26, [
                        first(15, 16), last(18, 19), reason(22, 26)
                    ]),
                    body(26, 300, [
                        test(27, 45, [
                            result(27, 29), number(30, 31), description(34, 45)
                        ]),
                        subtest(46, 295, [
                            name(57, 74),
                            plan(77, 89, [
                                first(77, 78), last(80, 81), reason(84, 89)
                            ]),
                            subtest(92, 273, [
                                plan(94, 106, [
                                    first(94, 95), last(97, 98), reason(101, 106)
                                ]),
                                test(111, 130, [
                                    result(111, 113), number(114, 115), description(118, 130)
                                ]),
                                anything(135, 236),
                                test(241, 272, [
                                    result(241, 247),
                                    number(248, 249),
                                    directive(250, 272, [
                                        key(252, 256),
                                        reason(257, 272)
                                    ])
                                ])
                            ]),
                            test(275, 294, [
                                result(275, 277), number(278, 279), description(282, 294)
                            ])
                        ]),
                        test(295, 299, [
                            result(295, 297), number(298, 299),
                        ]),
                    ])
                ])
            ]
        }
    }

    #[test]
    fn test_yaml() {
        let contents = fs::read_to_string("examples/yaml.tap").expect("Failed to read file");

        parses_to! {
            parser: TAPParser,
            input : &contents,
            rule: Rule::document,
            tokens: [
                document(0, 533, [
                    preamble(0, 14, [
                        version(12, 14)
                    ]),
                    plan(15, 31, [
                        first(15, 16), last(18, 19), reason(22, 31)
                    ]),
                    body(31, 533, [
                        test(32, 343, [
                            result(32, 38),
                            number(39, 40),
                            description(43, 58),
                            yaml_block(59, 343, [
                                yaml(67, 136),
                                yaml(139, 153),
                                yaml(156, 162),
                                yaml(165, 198),
                                yaml(201, 213),
                                yaml(216, 223),
                                yaml(226, 259),
                                yaml(262, 288),
                                yaml(291, 294),
                                yaml(297, 323),
                                yaml(326, 337),
                            ])
                        ]),
                        test(344, 365, [
                            result(344, 346), number(347, 348), description(351, 365)
                        ]),
                        subtest(366, 533, [
                            name(377, 401),
                            plan(404, 419, [
                                first(404, 405), last(407, 408), reason(411, 419)
                            ]),
                            test(422, 518, [
                                result(422, 424),
                                number(425, 426),
                                description(429, 443),
                                yaml_block(444, 518, [
                                    yaml(460, 508)
                                ]),
                            ]),
                            test(521, 532, [
                                result(521, 523), number(524, 525), description(528, 532)
                            ])
                        ])
                    ])
                ])
            ]
        }
    }
}
