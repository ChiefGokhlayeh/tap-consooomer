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
#[grammar = "tap14.pest"]
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
    fn test_common() {
        let contents = fs::read_to_string("examples/common.log").expect("Failed to read file");

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
        let contents = fs::read_to_string("examples/cascading.log").expect("Failed to read file");

        parses_to! {
            parser: TAPParser,
            input : &contents,
            rule: Rule::document,
            tokens: [
                document(0, 194, [
                    preamble(0, 14, [
                        version(12, 14)
                    ]),
                    plan(15, 26, [
                        first(15, 16), last(18, 19), reason(22, 26)
                    ]),
                    body(26, 194, [
                        test(27, 45, [
                            result(27, 29), number(30, 31), description(34, 45)
                        ]),
                        subtest(46, 189, [
                            name(57, 74),
                            plan(77, 89, [
                                first(77, 78), last(80, 81), reason(84, 89)
                            ]),
                            subtest(92, 167, [
                                plan(94, 106, [
                                    first(94, 95), last(97, 98), reason(101, 106)
                                ]),
                                test(111, 130, [
                                    result(111, 113), number(114, 115), description(118, 130)
                                ]),
                                test(135, 166, [
                                    result(135, 141),
                                    number(142, 143),
                                    directive(144, 166, [
                                        key(146, 150),
                                        reason(151, 166)
                                    ])
                                ])
                            ]),
                            test(169, 188, [
                                result(169, 171), number(172, 173), description(176, 188)
                            ])
                        ]),
                        test(189, 193, [
                            result(189, 191), number(192, 193),
                        ]),
                    ])
                ])
            ]
        }
    }
}
