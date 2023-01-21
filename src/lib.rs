extern crate pest;
#[macro_use]
extern crate pest_derive;

use anyhow::{anyhow, Result};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use serde::Serialize;

#[derive(Parser)]
#[grammar = "tap14.pest"]
pub struct TAPParser;

/// The TAP [`Preamble`] declares the start of a TAP document.
#[derive(Debug, Serialize)]
pub struct Preamble<'a> {
    /// TAP specification version. Can be any semantic version string (e.g. `14` or `14.1.3`).
    pub version: &'a str,
}

/// The [`Plan`] tells how many tests will be run, or how many tests have run.
#[derive(Debug, Serialize)]
pub struct Plan<'a> {
    /// ID of first planned test. _Should_ always start with `1`.
    pub first: i32,
    /// ID of last planned test. A value of `0` _should_ indicate no tests were executed.
    pub last: i32,
    /// Arbitrary string which _should_ indicate why the certain tests were skipped.
    pub reason: Option<&'a str>,
}

/// The body of the TAP document.
#[derive(Debug, Serialize)]
pub struct Body<'a> {
    /// List of [`Statement`]s.
    statements: Vec<Statement<'a>>,
}

/// A [`Pragma`] provides information to the document parser/interpreter.
///
/// # Note
///
/// Due to the PEG parsing approach, pragmas have no effect.
#[derive(Debug, Serialize)]
pub struct Pragma<'a> {
    /// If present, declares if the given `option` should be enabled or disabled.
    pub flag: Option<bool>,
    /// Pragma option identifier.
    pub option: &'a str,
}

/// Marks an emergency exit of the test procedure.
#[derive(Debug, Serialize)]
pub struct BailOut<'a> {
    /// Optional reason for bailing out of the test procedure.
    pub reason: Option<&'a str>,
}

/// Directive keys supported by [`Directive`].
#[derive(Debug, Serialize)]
pub enum Key {
    /// Test was skipped
    Skip,
    /// Test has a to-do.
    Todo,
}

/// A [`Directive`] gives some meta-data about the execution of a [`Test`].
#[derive(Debug, Serialize)]
pub struct Directive<'a> {
    /// A directive key, declaring the nature of this [`Directive`].
    pub key: Key,
    /// A reason why this test was [`Key::Skip`]ped or why it is a [`Key::Todo`].
    pub reason: Option<&'a str>,
}

/// A [`Test`] declaring the result of some test-case.
#[derive(Debug, Serialize)]
pub struct Test<'a> {
    /// Result of the test.
    pub result: bool,
    /// Number of the test.
    pub number: Option<i32>,
    /// Description of the test.
    pub description: Option<&'a str>,
    /// Directive detailing this tests meta-execution.
    pub directive: Option<Directive<'a>>,
    /// List of YAML lines detailing the test execution.
    pub yaml: Yaml<'a>,
}

/// [`Subtest`]s provide a way to nest one TAP14 stream inside another. This may be used in a variaty of ways, depending on
/// the test harness.
#[derive(Debug, Serialize)]
pub struct Subtest<'a> {
    /// Name of the subtest, declared by a comment at the start of the [`Subtest`].
    pub name: Option<&'a str>,
    /// The [`Plan`] of the [`Subtest`].
    pub plan: Plan<'a>,
    /// Main [`Body`] of the [`Subtest`].
    pub body: Vec<Statement<'a>>,
}

/// An enumeration of all possible TAP constructs that can be part of a [`Body`].
#[derive(Debug, Serialize)]
pub enum Statement<'a> {
    /// Any text not captured by another [`Statement`] variant.
    #[serde(rename = "anything")]
    Anything(&'a str),
    /// A [`BailOut`] statement.
    #[serde(rename = "bail_out")]
    BailOut(BailOut<'a>),
    /// A [`Pragma`] statement.
    #[serde(rename = "pragma")]
    Pragma(Pragma<'a>),
    /// A [`Subtest`] statement.
    #[serde(rename = "subtest")]
    Subtest(Subtest<'a>),
    /// A [`Test`] statement.
    #[serde(rename = "test")]
    Test(Test<'a>),
}

/// A [`Document`] represents the root of any TAP document. It's the main point of interaction for users of this API.
#[derive(Debug, Serialize)]
pub struct Document<'a> {
    /// The document's preamble.
    pub preamble: Preamble<'a>,
    /// The document's top-level plan declaration.
    pub plan: Plan<'a>,
    /// The document's top-level [`Body`] as a collection of [`Statement`]s. Some [`Statement`]s, like [`Subtest`] may
    /// declare _nested_ [`Body`]s.
    pub body: Vec<Statement<'a>>,
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

type Yaml<'a> = Vec<&'a str>;

impl<'a> Preamble<'a> {
    fn parse(mut pairs: Pairs<'a, Rule>) -> Self {
        Self {
            version: pairs.next().unwrap().as_str(),
        }
    }

    /// Parse [`Preamble`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP version 14 preamble may look like this:
    ///
    /// ```
    /// use tapconsooomer::Preamble;
    ///
    /// let content = "TAP version 14";
    /// let preamble = Preamble::parse_from_str(content).expect("Parser error");
    /// assert_eq!(preamble.version, "14");
    /// ```
    ///
    /// Semantic versioning is supported aswell:
    ///
    /// ```
    /// use tapconsooomer::Preamble;
    ///
    /// let content = "TAP version 13.1";
    /// let preamble = Preamble::parse_from_str(content).expect("Parser error");
    /// assert_eq!(preamble.version, "13.1");
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::preamble, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))
    }
}

impl<'a> Plan<'a> {
    fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
        Ok(Self {
            first: pairs.next().unwrap().as_str().parse()?,
            last: pairs.next().unwrap().as_str().parse()?,
            reason: pairs.next().map(|r| r.as_str()),
        })
    }

    /// Parse [`Plan`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP plan may look like this:
    ///
    /// ```
    /// use tapconsooomer::Plan;
    ///
    /// let content = "1..5 # TODO: Test 3 may fail";
    /// let plan = Plan::parse_from_str(content).expect("Parser error");
    /// assert_eq!(plan.first, 1);
    /// assert_eq!(plan.last, 5);
    /// assert_eq!(plan.reason, Some("TODO: Test 3 may fail"));
    /// ```
    ///
    /// Note, [`Plan::reason`] is optional:
    ///
    /// ```
    /// use tapconsooomer::Plan;
    ///
    /// let content = "1..5";
    /// let plan = Plan::parse_from_str(content).expect("Parser error");
    /// assert_eq!(plan.first, 1);
    /// assert_eq!(plan.last, 5);
    /// assert_eq!(plan.reason, None);
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::plan, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> Directive<'a> {
    fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
        let key = pairs.next().unwrap().as_str().to_lowercase();
        Ok(Self {
            key: match key.as_str() {
                "skip" => Ok(Key::Skip),
                "todo" => Ok(Key::Todo),
                _ => Err(anyhow!("Directive key '{}' must be 'skip' or 'todo'", key)),
            }?,
            reason: pairs.next().map(|p| p.as_str()),
        })
    }

    /// Parse [`Directive`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP directive may look like this:
    ///
    /// ```
    /// use tapconsooomer::Directive;
    /// use tapconsooomer::Key;
    ///
    /// let content = "# SKIP hardware requirements not met";
    /// let directive = Directive::parse_from_str(content).expect("Parser error");
    /// assert!(matches!(directive.key, Key::Skip));
    /// assert_eq!(directive.reason, Some("hardware requirements not met"));
    /// ```
    ///
    /// Note, [`Directive::reason`] is optional:
    ///
    /// ```
    /// use tapconsooomer::Directive;
    /// use tapconsooomer::Key;
    ///
    /// let content = "# TODO";
    /// let directive = Directive::parse_from_str(content).expect("Parser error");
    /// assert!(matches!(directive.key, Key::Todo));
    /// assert_eq!(directive.reason, None);
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::directive, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> Test<'a> {
    fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
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
                Rule::directive => directive = Directive::parse(pair.into_inner()).ok(),
                Rule::yaml_block => {
                    yaml.append(&mut { pair.into_inner().map(|p| p.as_str()).collect() })
                }
                _ => unreachable!(),
            };
        }
        Ok(Self {
            result,
            number,
            description,
            directive,
            yaml,
        })
    }

    /// Parse [`Test`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP test may look like this:
    ///
    /// ```
    /// use tapconsooomer::Test;
    ///
    /// let content = "not ok 1 - foo()";
    /// let test = Test::parse_from_str(content).expect("Parser error");
    /// assert_eq!(test.result, false);
    /// assert_eq!(test.number, Some(1));
    /// assert_eq!(test.description, Some("foo()"));
    /// assert_eq!(test.directive.is_none(), true);
    /// assert_eq!(test.yaml.len(), 0);
    /// ```
    ///
    /// Note, many attributes of a TAP test are optional. A minimal TAP test may look like this:
    ///
    /// ```
    /// use tapconsooomer::Test;
    ///
    /// let content = "ok";
    /// let test = Test::parse_from_str(content).expect("Parser error");
    /// assert_eq!(test.result, true);
    /// assert_eq!(test.number, None);
    /// assert_eq!(test.description, None);
    /// assert_eq!(test.directive.is_none(), true);
    /// assert_eq!(test.yaml.len(), 0);
    /// ```
    ///
    /// TAP tests may also optionally contain a YAML block. While no parsing of actual YAML syntax is performed, the
    /// parser captures each line inside the YAML block in a [`Vec`]:
    ///
    /// ```
    /// use tapconsooomer::Test;
    ///
    /// let content = concat!(
    ///     "not ok 2 - bar()\n",
    ///     "  ---\n",
    ///     "  message: invalid input\n",
    ///     "  status: failed\n",
    ///     "  ...\n",
    /// );
    /// let test = Test::parse_from_str(content).expect("Parser error");
    /// assert_eq!(test.result, false);
    /// assert_eq!(test.number, Some(2));
    /// assert_eq!(test.description, Some("bar()"));
    /// assert_eq!(test.directive.is_none(), true);
    /// assert_eq!(test.yaml.len(), 2);
    /// assert_eq!(test.yaml, ["message: invalid input", "status: failed"])
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::test, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> BailOut<'a> {
    fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
        Ok(Self {
            reason: pairs.next().map(|p| p.as_str()),
        })
    }

    /// Parse [`BailOut`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP bail-out may look like this:
    ///
    /// ```
    /// use tapconsooomer::BailOut;
    ///
    /// let content = "Bail out! Hardware overheating";
    /// let bail_out = BailOut::parse_from_str(content).expect("Parser error");
    /// assert_eq!(bail_out.reason, Some("Hardware overheating"));
    /// ```
    ///
    /// Note, [`BailOut::reason`] is optional:
    ///
    /// ```
    /// use tapconsooomer::BailOut;
    ///
    /// let content = "Bail out!";
    /// let bail_out = BailOut::parse_from_str(content).expect("Parser error");
    /// assert_eq!(bail_out.reason.is_none(), true);
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::bail_out, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> Pragma<'a> {
    pub fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
        let mut pair = pairs.next().unwrap();
        let flag = match pair.as_rule() {
            Rule::flag => {
                println!("{}", pair.as_str());
                let result = match pair.as_str() {
                    "+" => Some(true),
                    "-" => Some(false),
                    _ => unreachable!(),
                };
                pair = pairs.next().unwrap();
                result
            }
            _ => None,
        };
        Ok(Self {
            flag,
            option: pair.as_str(),
        })
    }

    /// Parse [`Pragma`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP pragma may look like this:
    ///
    /// ```
    /// use tapconsooomer::Pragma;
    ///
    /// let content = "pragma +strict";
    /// let pragma = Pragma::parse_from_str(content).expect("Parser error");
    /// assert_eq!(pragma.flag, Some(true));
    /// assert_eq!(pragma.option, "strict");
    /// ```
    ///
    /// Note, [`Pragma::flag`] is optional:
    ///
    /// ```
    /// use tapconsooomer::Pragma;
    ///
    /// let content = "pragma foo";
    /// let pragma = Pragma::parse_from_str(content).expect("Parser error");
    /// assert_eq!(pragma.flag.is_none(), true);
    /// assert_eq!(pragma.option, "foo");
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::pragma, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> Subtest<'a> {
    pub fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
        let pair = pairs.next().unwrap();
        let name = match pair.as_rule() {
            Rule::name => Some(pair.as_str()),
            _ => None,
        };

        let mut contents = vec![]; /* FIXME: There has to be a better way to split plan and body! */

        /* Consume first pair if not consumed by 'name'. */
        if name.is_none() {
            contents.push(match pair.as_rule() {
                Rule::plan => Plan::parse(pair.into_inner()).map(SubtestContent::Plan),
                _ => Statement::parse(pair).map(SubtestContent::Statement),
            }?)
        }

        /* Now consume the rest of the pairs. */
        let mut rest: Vec<SubtestContent> = pairs
            .map(|p| match p.as_rule() {
                Rule::plan => Plan::parse(p.into_inner()).map(SubtestContent::Plan),
                _ => Statement::parse(p).map(SubtestContent::Statement),
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

        Ok(Self { name, plan, body })
    }

    /// Parse [`Subtest`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP subtest may look like this:
    ///
    /// ```
    /// use tapconsooomer::Subtest;
    ///
    /// let content = concat!(
    ///     "# Subtest: foo\n",
    ///     "  1..2\n",
    ///     "  ok 1 - bar\n",
    ///     "  ok 2 - tuna\n",
    /// );
    /// let subtest = Subtest::parse_from_str(content).expect("Parser error");
    /// assert_eq!(subtest.name, Some("foo"));
    /// assert_eq!(subtest.plan.first, 1);
    /// assert_eq!(subtest.plan.last, 2);
    /// assert_eq!(subtest.body.len(), 2);
    /// ```
    ///
    /// Note, the comment declaring [`Subtest::name`] is optional:
    ///
    /// ```
    /// use tapconsooomer::Subtest;
    ///
    /// let content = concat!(
    ///     "  1..4\n",
    ///     "  ok 1 - foo\n",
    ///     "  ok 2 - bar\n",
    ///     "  ok 3 - foobar\n",
    ///     "  ok 4 - catfood\n",
    /// );
    /// let subtest = Subtest::parse_from_str(content).expect("Parser error");
    /// assert_eq!(subtest.name.is_none(), true);
    /// assert_eq!(subtest.plan.first, 1);
    /// assert_eq!(subtest.plan.last, 4);
    /// assert_eq!(subtest.body.len(), 4);
    /// ```
    ///
    /// So is the order in which [`Plan`] and [`Body`] are declared:
    ///
    /// ```
    /// use tapconsooomer::Subtest;
    ///
    /// let content = concat!(
    ///     "  ok 1 - hello world\n",
    ///     "  1..1\n",
    /// );
    /// let subtest = Subtest::parse_from_str(content).expect("Parser error");
    /// assert_eq!(subtest.name.is_none(), true);
    /// assert_eq!(subtest.plan.first, 1);
    /// assert_eq!(subtest.plan.last, 1);
    /// assert_eq!(subtest.body.len(), 1);
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::subtest, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> Statement<'a> {
    pub fn parse(pair: Pair<'a, Rule>) -> Result<Self> {
        match pair.as_rule() {
            Rule::test => Ok(Self::Test(Test::parse(pair.into_inner())?)),
            Rule::bail_out => Ok(Self::BailOut(BailOut::parse(pair.into_inner())?)),
            Rule::pragma => Ok(Self::Pragma(Pragma::parse(pair.into_inner())?)),
            Rule::subtest => Ok(Self::Subtest(Subtest::parse(pair.into_inner())?)),
            Rule::anything => Ok(Self::Anything(pair.as_str())),
            _ => unreachable!(),
        }
    }

    /// Parse [`Statement`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP statement may look like this:
    ///
    /// ```
    /// use tapconsooomer::Statement;
    ///
    /// let test = "ok 1 - foo::bar()";
    /// let stmt = Statement::parse_from_str(test).expect("Parser error");
    /// assert!(matches!(stmt, Statement::Test(..)));
    ///
    /// let anything = "hello world";
    /// let stmt = Statement::parse_from_str(anything).expect("Parser error");
    /// assert!(matches!(stmt, Statement::Anything(..)));
    ///
    /// let bail_out = "Bail Out!";
    /// let stmt = Statement::parse_from_str(bail_out).expect("Parser error");
    /// assert!(matches!(stmt, Statement::BailOut(..)));
    ///
    /// let pragma = "pragma -strict";
    /// let stmt = Statement::parse_from_str(pragma).expect("Parser error");
    /// assert!(matches!(stmt, Statement::Pragma(..)));
    ///
    /// let subtest = concat!(
    ///     "# Subtest: foo\n",
    ///     "    1..0 # skipped\n",
    /// );
    /// let stmt = Statement::parse_from_str(subtest).expect("Parser error");
    /// assert!(matches!(stmt, Statement::Subtest(..)));
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::statement, content)?
            .next()
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

impl<'a> DocumentContent<'a> {
    fn parse(pair: Pair<'a, Rule>) -> Result<Self> {
        Ok(match pair.as_rule() {
            Rule::plan => Self::Plan(Plan::parse(pair.into_inner())?),
            Rule::body => {
                let statements: Result<Vec<_>> = pair.into_inner().map(Statement::parse).collect();
                Self::Body(statements?)
            }
            _ => unreachable!(),
        })
    }
}

impl<'a> Document<'a> {
    pub fn parse(mut pairs: Pairs<'a, Rule>) -> Result<Self> {
        let preamble = Preamble::parse(pairs.next().unwrap().into_inner());

        let content1 = DocumentContent::parse(pairs.next().unwrap())?;
        let content2 = DocumentContent::parse(pairs.next().unwrap())?;
        let (plan, body) = match (content1, content2) {
            (DocumentContent::Plan(p), DocumentContent::Body(b)) => (p, b),
            (DocumentContent::Body(b), DocumentContent::Plan(p)) => (p, b),
            _ => unreachable!(),
        };

        Ok(Self {
            preamble,
            plan,
            body,
        })
    }

    /// Parse [`Document`] from a `&str`.
    ///
    /// # Examples
    ///
    /// Parsing a TAP document may look like this:
    ///
    /// ```
    /// use tapconsooomer::Document;
    ///
    /// let content = concat!(
    ///     "TAP version 14\n",
    ///     "1..1\n",
    ///     "ok 1 - foo()\n",
    /// );
    /// let doc = Document::parse_from_str(content).expect("Parser error");
    /// assert_eq!(doc.preamble.version, "14");
    /// assert_eq!(doc.body.len(), 1);
    /// ```
    ///
    /// The order in which [`Body`] and [`Plan`] are declared is unimportant:
    ///
    /// ```
    /// use tapconsooomer::Document;
    ///
    /// let content = concat!(
    ///     "TAP version 14\n",
    ///     "ok 1 - foo()\n",
    ///     "ok 2 - bar()\n",
    ///     "1..2\n",
    /// );
    /// let doc = Document::parse_from_str(content).expect("Parser error");
    /// assert_eq!(doc.preamble.version, "14");
    /// assert_eq!(doc.body.len(), 2);
    /// ```
    pub fn parse_from_str(content: &'a str) -> Result<Self> {
        TAPParser::parse(Rule::document, content)?
            .next()
            .map(Pair::into_inner)
            .map(Self::parse)
            .ok_or_else(|| anyhow!("Can't parse '{}'", content))?
    }
}

#[cfg(test)]
mod tests {
    use pest::consumes_to;
    use pest::parses_to;
    use std::fs;

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
                    flag(7, 8), option(8, 14)
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
                    flag(7, 8), option(8, 14)
                ])
            ]
        }
    }

    #[test]
    fn test_pragma_no_flag() {
        parses_to! {
            parser: TAPParser,
            input : "pragma allow_anything",
            rule: Rule::pragma,
            tokens: [
                pragma(0, 21, [
                    option(7, 21)
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
                "        \r\n",
                "  0..1 # even this is YAML\n",
                "  ...\n",
            ),
            rule: Rule::yaml_block,
            tokens: [
                yaml_block(0, 123, [
                    yaml(8, 19),
                    yaml(22, 33),
                    yaml(36, 47),
                    yaml(50, 80),
                    yaml(93, 117),
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
                document(0, 534, [
                    preamble(0, 14, [
                        version(12, 14)
                    ]),
                    plan(15, 31, [
                        first(15, 16), last(18, 19), reason(22, 31)
                    ]),
                    body(31, 534, [
                        test(32, 344, [
                            result(32, 38),
                            number(39, 40),
                            description(43, 58),
                            yaml_block(59, 344, [
                                yaml(67, 136),
                                yaml(139, 153),
                                yaml(156, 162),
                                yaml(165, 198),
                                yaml(201, 213),
                                yaml(217, 224),
                                yaml(227, 260),
                                yaml(263, 289),
                                yaml(292, 295),
                                yaml(298, 324),
                                yaml(327, 338),
                            ])
                        ]),
                        test(345, 366, [
                            result(345, 347), number(348, 349), description(352, 366)
                        ]),
                        subtest(367, 534, [
                            name(378, 402),
                            plan(405, 420, [
                                first(405, 406), last(408, 409), reason(412, 420)
                            ]),
                            test(423, 519, [
                                result(423, 425),
                                number(426, 427),
                                description(430, 444),
                                yaml_block(445, 519, [
                                    yaml(461, 509)
                                ]),
                            ]),
                            test(522, 533, [
                                result(522, 524), number(525, 526), description(529, 533)
                            ])
                        ])
                    ])
                ])
            ]
        }
    }
}
