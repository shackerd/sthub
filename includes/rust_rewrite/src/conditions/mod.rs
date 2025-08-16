use std::str::FromStr;

pub mod context;
mod error;
mod matcher;
mod parse;

use matcher::{Match, Value};

pub use context::EngineCtx;
pub use error::CondError;

/// Singular `RewriteCond` expression definition.
///
/// It contains an expression matcher and additional flags that
/// define how the rule behaves within the rule-engine.
///
/// Supports a subset of [offical](https://httpd.apache.org/docs/current/mod/mod_rewrite.html#rewritecond)
/// mod_rewrite rules.
#[derive(Clone, Debug)]
pub struct Condition {
    matcher: Match,
    flags: Vec<CondFlag>,
}

impl Condition {
    /// Evaluate if the rewrite condition and return boolean result.
    pub fn is_met(&self, ctx: &mut EngineCtx) -> bool {
        let nocase = self.flags.iter().any(|f| matches!(f, CondFlag::NoCase));
        match &self.matcher {
            Match::Pattern(v1, pt, v2) => {
                pt.matches(Value::new(v1, nocase, ctx), Value::new(v2, nocase, ctx))
            }
            Match::NotPattern(v1, pt, v2) => {
                !pt.matches(Value::new(v1, nocase, ctx), Value::new(v2, nocase, ctx))
            }
            Match::Compare(v1, cp, v2) => {
                cp.compare(Value::new(v1, nocase, ctx), Value::new(v2, nocase, ctx))
            }
            Match::FileTest(v1, ft) => ft.matches(Value::new(v1, nocase, ctx)),
            Match::NotFileTest(v1, ft) => !ft.matches(Value::new(v1, nocase, ctx)),
        }
    }

    /// Returns true if the rewrite condition uses OR operator rather
    /// than the default AND.
    #[inline]
    pub fn is_or(&self) -> bool {
        self.flags.iter().any(|c| matches!(c, CondFlag::Or))
    }
}

impl FromStr for Condition {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = parse::tokenize(s)?.into_iter().peekable();
        let matcher = Match::parse(&mut tokens)?;
        let flags = match tokens.next() {
            Some(flags) => CondFlagList::from_str(&flags)?.0,
            None => Vec::new(),
        };
        Ok(Self { matcher, flags })
    }
}

struct CondFlagList(Vec<CondFlag>);

impl FromStr for CondFlagList {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('[') || !s.ends_with(']') {
            return Err(CondError::FlagsMissingBrackets(s.to_owned()));
        }
        let flags = s[1..s.len() - 1]
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(CondFlag::from_str)
            .collect::<Result<Vec<CondFlag>, _>>()?;
        if flags.is_empty() {
            return Err(CondError::FlagsEmpty);
        }
        Ok(Self(flags))
    }
}

/// Supported `mod_rewrite` [`Condition`] flags that modify
/// the conditions behavior.
#[derive(Clone, Debug)]
enum CondFlag {
    NoCase,
    Or,
}

impl FromStr for CondFlag {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "i" | "insensitive" | "nc" | "nocase" => Ok(Self::NoCase),
            "or" | "ornext" => Ok(Self::Or),
            _ => Err(CondError::InvalidFlag(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use context::{RequestCtx, ServerCtx};
    use matcher::{Compare, FileTest, Pattern};

    #[test]
    fn test_pattern() {
        let s1 = String::from("%{REQUEST_URI}");
        let s2 = String::from("/Test");
        let cond = Condition::from_str(&format!(r#"{s1} "={s2}" [NC,OR]"#)).unwrap();
        assert!(matches!(
            &cond.matcher,
            Match::Pattern(v1, Pattern::Equals, v2) if v1 == &s1 && v2 == &s2,
        ));
        assert_eq!(cond.flags.len(), 2);
        assert!(matches!(cond.flags.get(0), Some(CondFlag::NoCase)));

        let mut req = RequestCtx::default().request_uri("/Test");
        let mut ctx = EngineCtx::default().with_ctx(req);
        assert!(cond.is_met(&mut ctx));

        req = RequestCtx::default().request_uri("/Not");
        let mut ctx = EngineCtx::default().with_ctx(req);
        assert!(!cond.is_met(&mut ctx));
    }

    #[test]
    fn test_compare() {
        let s1 = String::from("%{SERVER_PORT}");
        let s2 = String::from("4000");
        let cond = Condition::from_str(&format!("{s1} -ge {s2}")).unwrap();
        assert!(matches!(
            &cond.matcher,
            Match::Compare(v1, Compare::GreaterOrEqual, v2) if v1 == &s1 && v2 == &s2,
        ));
        assert_eq!(cond.flags.len(), 0);

        let mut srv = ServerCtx::default().server_addr("127.0.0.1:4001").unwrap();
        let mut ctx = EngineCtx::default().with_ctx(srv);
        assert!(cond.is_met(&mut ctx));

        srv = ServerCtx::default().server_addr("127.0.0.1:3999").unwrap();
        let mut ctx = EngineCtx::default().with_ctx(srv);
        assert!(!cond.is_met(&mut ctx));
    }

    #[test]
    fn test_filetest() {
        let s1 = String::from("%{REQUEST_URI}");
        let cond = Condition::from_str(&format!("{s1} !-f")).unwrap();
        assert!(matches!(
            &cond.matcher,
            Match::NotFileTest(v1, FileTest::File) if v1 == &s1,
        ));
        assert_eq!(cond.flags.len(), 0);

        let current = std::env::current_dir().unwrap().join("src").join("lib.rs");
        let mut req = RequestCtx::default().request_uri(current.to_str().unwrap());
        let mut ctx = EngineCtx::default().with_ctx(req);
        assert!(!cond.is_met(&mut ctx));

        req = RequestCtx::default().request_uri("/invalid");
        let mut ctx = EngineCtx::default().with_ctx(req);
        assert!(cond.is_met(&mut ctx));
    }
}
