use std::iter::Peekable;
use std::ops::Deref;
use std::{os::unix::fs::PermissionsExt, path::PathBuf, str::FromStr};

use unicase::UniCase;

use super::context::EngineCtx;
use super::error::CondError;
use super::parse::*;

/// Abstraction for String value that supports toggling case
/// insensitivity when evaluating [`Match`] comparisons
#[derive(Debug, PartialEq)]
pub enum Value {
    NoCase(UniCase<String>),
    Case(String),
}

impl Deref for Value {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::NoCase(uni) => uni.deref(),
            Self::Case(s) => s,
        }
    }
}

impl Value {
    /// Build new [`Value`] instance with the configured settings.
    ///
    /// Replaces all variables using [`EngineCtx`] before configuring
    /// for case-sensitive settings.
    pub fn new(s: &str, nocase: bool, ctx: &mut EngineCtx) -> Self {
        let value = ctx.replace_all(s);
        match nocase {
            true => Self::NoCase(UniCase::new(value)),
            false => Self::Case(value),
        }
    }

    /// [`String::starts_with`](std::string::String) abstraction.
    pub fn starts_with(&self, s: &Value) -> bool {
        match self {
            Self::Case(c) => c.starts_with(s.deref()),
            Self::NoCase(c) => c.starts_with(s.deref()),
        }
    }

    /// [`String::starts_with`](std::string::String) abstraction.
    pub fn ends_with(&self, s: &Value) -> bool {
        match self {
            Self::Case(c) => c.ends_with(s.deref()),
            Self::NoCase(c) => c.ends_with(s.deref()),
        }
    }
}

/// Compiled condition logical expression.
///
/// Supports `CondPattern`, integer comparisons, and file attribute tests
/// with negated variations.
#[derive(Clone, Debug, PartialEq)]
pub enum Match {
    Pattern(String, Pattern, String),
    NotPattern(String, Pattern, String),
    Compare(String, Compare, String),
    FileTest(String, FileTest),
    NotFileTest(String, FileTest),
}

impl Match {
    pub(crate) fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, CondError>
    where
        I: Iterator<Item = String>,
    {
        let first = tokens.next().ok_or(CondError::EmptyExpression)?;
        let expr = tokens.next().ok_or(CondError::MissingComparison)?;

        let not = expr.starts_with('!');
        let expr = expr.trim_start_matches('!');
        if let Some(c) = matches_start(expr, &['<', '>', '=']) {
            let (_, second) = expr
                .split_once(c)
                .ok_or(CondError::InvalidPattern(expr.to_owned()))?;
            let pattern = Pattern::from_str(&expr[..expr.len() - second.len()])?;
            return match not {
                true => Ok(Self::NotPattern(first, pattern, second.to_owned())),
                false => Ok(Self::Pattern(first, pattern, second.to_owned())),
            };
        }

        let second = tokens.peek();
        if second.is_some_and(|s| !s.starts_with('[')) {
            let second = tokens.next().unwrap();
            if not {
                return Err(CondError::InvalidComparison(expr.to_owned()));
            }
            let cmp = Compare::from_str(expr)?;
            return Ok(Self::Compare(first, cmp, second.to_owned()));
        }

        let ftest = FileTest::from_str(expr)?;
        match not {
            true => Ok(Self::NotFileTest(first, ftest)),
            false => Ok(Self::FileTest(first, ftest)),
        }
    }
}

impl FromStr for Match {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = tokenize(s)?.into_iter().peekable();
        Self::parse(&mut tokens)
    }
}

/// `CondPattern` expression definition.
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Preceeds,
    Follows,
    Equals,
    PreceedsOrEquals,
    FollowsOrEquals,
}

impl Pattern {
    /// Evaluate `CondPattern` according to defintion.
    pub fn matches(&self, first: Value, second: Value) -> bool {
        match self {
            Self::Preceeds | Self::PreceedsOrEquals => first.starts_with(&second),
            Self::Follows | Self::FollowsOrEquals => first.ends_with(&second),
            Self::Equals => first == second,
        }
    }
}

impl FromStr for Pattern {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(Self::Preceeds),
            ">" => Ok(Self::Follows),
            "=" => Ok(Self::Equals),
            "<=" => Ok(Self::PreceedsOrEquals),
            ">=" => Ok(Self::FollowsOrEquals),
            _ => Err(CondError::InvalidPattern(s.to_owned())),
        }
    }
}

/// Integer comparison expression definition.
#[derive(Clone, Debug, PartialEq)]
pub enum Compare {
    Equal,
    GreaterThan,
    GreaterOrEqual,
    LesserThan,
    LesserOrEqual,
    NotEqual,
}

impl Compare {
    /// Evaluate integer expression according to definition.
    pub fn compare(&self, first: Value, second: Value) -> bool {
        let Some(first) = first.parse::<i32>().ok() else {
            return false;
        };
        let Some(second) = second.parse::<i32>().ok() else {
            return false;
        };
        match self {
            Self::Equal => first == second,
            Self::GreaterThan => first > second,
            Self::GreaterOrEqual => first >= second,
            Self::LesserThan => first < second,
            Self::LesserOrEqual => first <= second,
            Self::NotEqual => first != second,
        }
    }
}

impl FromStr for Compare {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-eq" => Ok(Self::Equal),
            "-gt" => Ok(Self::GreaterThan),
            "-ge" => Ok(Self::GreaterOrEqual),
            "-lt" => Ok(Self::LesserThan),
            "-le" => Ok(Self::LesserOrEqual),
            "-ne" => Ok(Self::NotEqual),
            _ => Err(CondError::InvalidComparison(s.to_owned())),
        }
    }
}

/// File attribute-test expression definition.
#[derive(Clone, Debug, PartialEq)]
pub enum FileTest {
    Dir,
    File,
    Symbolic,
    SizedFile,
    Executable,
}

impl FileTest {
    /// Evaluate file attribute-test according to defintion.
    pub fn matches(&self, path: Value) -> bool {
        let path = PathBuf::from(path.deref());
        match self {
            Self::Dir => path.is_dir(),
            Self::File => path.is_file(),
            Self::Symbolic => path.is_symlink(),
            Self::SizedFile => {
                path.is_file() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
            }
            Self::Executable => path
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false),
        }
    }
}

impl FromStr for FileTest {
    type Err = CondError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-d" => Ok(Self::Dir),
            "-f" => Ok(Self::File),
            "-h" | "-l" => Ok(Self::Symbolic),
            "-s" => Ok(Self::SizedFile),
            "-x" => Ok(Self::Executable),
            _ => Err(CondError::InvalidFileTest(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern() {
        assert_eq!(
            Match::from_str(r#" %{REQUEST_URI} "=/this/test" "#).ok(),
            Some(Match::Pattern(
                String::from("%{REQUEST_URI}"),
                Pattern::Equals,
                String::from("/this/test")
            ))
        );
    }

    #[test]
    fn test_compare() {
        assert_eq!(
            Match::from_str(r#" %{REMOTE_PORT} -eq 4000 "#).ok(),
            Some(Match::Compare(
                String::from("%{REMOTE_PORT}"),
                Compare::Equal,
                String::from("4000"),
            ))
        );
        assert!(matches!(
            Match::from_str(r#"%{REMOTE_PORT} -wtf 4000 "#).err(),
            Some(CondError::InvalidComparison(_))
        ));
    }

    #[test]
    fn test_filetest() {
        assert_eq!(
            Match::from_str(r#" /var/www/%{REQUEST_URI} !-f "#).ok(),
            Some(Match::NotFileTest(
                String::from("/var/www/%{REQUEST_URI}"),
                FileTest::File,
            ))
        );
        assert!(matches!(
            Match::from_str(r#" /var/www/%{REQUEST_URI} !-A "#).err(),
            Some(CondError::InvalidFileTest(_))
        ));
    }
}
