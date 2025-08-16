use std::str::FromStr;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use regex_automata::{
    MatchKind,
    meta::{self, Regex},
    util,
};

use super::error::RuleError;

// https://url.spec.whatwg.org/#percent-encoded-bytes
const ESCAPE: &AsciiSet = &CONTROLS
    .add(b'~')
    .add(b' ') // fragment encoding
    .add(b'\'')
    .add(b'"')
    .add(b'`')
    .add(b'#') // query encoding
    .add(b'<')
    .add(b'>')
    .add(b'?') // path encoding
    .add(b'^')
    .add(b'{')
    .add(b'}')
    .add(b'/') // user-info encoding
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b']')
    .add(b'$') // component encoding
    .add(b'&')
    .add(b'+')
    .add(b',');

/// Singular `RewriteRule` expression definition.
///
/// It contains a regex pattern to match against a request uri,
/// a rewrite string that expands into the new uri, and additional
/// flags that define how the rule behaves within the rule-engine.
///
/// Supports a subset of [offical](https://httpd.apache.org/docs/current/mod/mod_rewrite.html#rewriterule)
/// mod_rewrite rules.
#[derive(Clone, Debug)]
pub struct Rule {
    pattern: Regex,
    rewrite: String,
    flags: Vec<RuleFlag>,
}

impl Rule {
    /// Try to match the rewrite expression pattern to the specified uri.
    ///
    /// Produces a new re-written string if the rewrite rule matched.
    #[inline]
    pub fn try_rewrite(&self, uri: &str) -> Option<String> {
        let mut caps = self.pattern.create_captures();
        self.pattern.captures(uri, &mut caps);
        if !caps.is_match() {
            return None;
        }

        let noescape = self
            .flags
            .iter()
            .any(|f| matches!(f, RuleFlag::Mod(RuleMod::NoEscape)));

        let mut dst = String::new();
        util::interpolate::string(
            &self.rewrite,
            |index, dst| {
                let string = match caps.get_group(index) {
                    None => return,
                    Some(span) => &uri[span],
                };
                if noescape {
                    return dst.push_str(string);
                }
                let s = utf8_percent_encode(string, ESCAPE).to_string();
                dst.push_str(&s);
            },
            |name| caps.group_info().to_index(caps.pattern()?, name),
            &mut dst,
        );
        Some(dst)
    }

    /// Retrieves the associated [`RuleShift`] defined in the
    /// expressions flags if any is present.
    #[inline]
    pub(crate) fn shift(&self) -> Option<&RuleShift> {
        self.flags.iter().find_map(|f| match f {
            RuleFlag::Shift(shift) => Some(shift),
            _ => None,
        })
    }

    /// Retrieve the associated [`RuleResolve`] defined in the
    /// expressions flags if any is present.
    #[inline]
    pub(crate) fn resolve(&self) -> Option<&RuleResolve> {
        self.flags.iter().find_map(|f| match f {
            RuleFlag::Resolve(resolve) => Some(resolve),
            _ => None,
        })
    }
}

impl FromStr for Rule {
    type Err = RuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = s.split_whitespace().filter(|s| !s.is_empty());
        let pattern = items.next().ok_or(RuleError::MissingPattern)?;
        let rewrite = items.next().ok_or(RuleError::MissingRewrite)?.to_string();
        let flags = match items.next() {
            Some(flags) => RuleFlagList::from_str(flags)?.0,
            None => Vec::new(),
        };
        if let Some(next) = items.next() {
            return Err(RuleError::InvalidSuffix(next.to_owned()));
        }

        let insense = flags
            .iter()
            .any(|f| matches!(f, RuleFlag::Mod(RuleMod::NoCase)));
        let regex = Regex::builder()
            .configure(
                meta::Config::new()
                    .nfa_size_limit(Some(10 * (1 << 20)))
                    .hybrid_cache_capacity(2 * (1 << 20))
                    .match_kind(MatchKind::LeftmostFirst)
                    .utf8_empty(true),
            )
            .syntax(util::syntax::Config::new().case_insensitive(insense))
            .build(pattern)
            .map_err(|err| RuleError::InvalidRegex(err.to_string()))?;

        Ok(Self {
            pattern: regex,
            rewrite,
            flags,
        })
    }
}

struct RuleFlagList(Vec<RuleFlag>);

impl FromStr for RuleFlagList {
    type Err = RuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('[') || !s.ends_with(']') {
            return Err(RuleError::FlagsMissingBrackets(s.to_owned()));
        }
        let flags = s[1..s.len() - 1]
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(RuleFlag::from_str)
            .collect::<Result<Vec<RuleFlag>, _>>()?;
        if flags.is_empty() {
            return Err(RuleError::FlagsEmpty);
        }
        let num_meta = flags
            .iter()
            .filter(|f| matches!(f, RuleFlag::Shift(_)))
            .count();
        let num_response = flags
            .iter()
            .filter(|f| matches!(f, RuleFlag::Resolve(_)))
            .count();
        if (num_meta + num_response) > 1 {
            return Err(RuleError::FlagsMutuallyExclusive);
        }
        Ok(Self(flags))
    }
}

#[inline]
fn parse_int(s: &str, default: u16) -> Result<u16, RuleError> {
    match s.is_empty() {
        true => Ok(default),
        false => Ok(u16::from_str(s)?),
    }
}

#[inline]
fn parse_status(s: &str, default: u16) -> Result<u16, RuleError> {
    let status = parse_int(s, default)?;
    match !(100..600).contains(&status) {
        true => Err(RuleError::InvalidFlagStatus(s.to_owned())),
        false => Ok(status),
    }
}

/// [`RuleFlag`] subtype declaring shift in rule processing after match
#[derive(Clone, Debug)]
pub enum RuleShift {
    End,
    Last,
    Next,
    Skip(u16),
}

/// [`RuleFlag`] subtype declaring a modification in rewrite behavior
#[derive(Clone, Debug)]
pub enum RuleMod {
    NoCase,
    NoEscape,
}

/// [`RuleFlag`] subtype declaring a final http-response resolution
#[derive(Clone, Debug)]
pub enum RuleResolve {
    Redirect(u16),
    Status(u16),
}

/// Flag Modifiers to a [`Rule`] expression.
///
/// Supports a subset of [official](https://httpd.apache.org/docs/current/rewrite/flags.html)
/// `mod_rewrite` flags.
#[derive(Clone, Debug)]
pub enum RuleFlag {
    Shift(RuleShift),
    Mod(RuleMod),
    Resolve(RuleResolve),
}

impl FromStr for RuleFlag {
    type Err = RuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (p, s) = match s.split_once('=') {
            Some((prefix, suffix)) => (prefix, suffix),
            None => (s, ""),
        };
        match p.to_lowercase().as_str() {
            "e" | "end" => Ok(Self::Shift(RuleShift::End)),
            "l" | "last" => Ok(Self::Shift(RuleShift::Last)),
            "n" | "next" => Ok(Self::Shift(RuleShift::Next)),
            "s" | "skip" => Ok(Self::Shift(RuleShift::Skip(parse_int(s, 1)?))),
            "i" | "insensitive" | "nc" | "nocase" => Ok(Self::Mod(RuleMod::NoCase)),
            "ne" | "noescape" => Ok(Self::Mod(RuleMod::NoEscape)),
            "r" | "redirect" => Ok(Self::Resolve(RuleResolve::Redirect(parse_status(s, 302)?))),
            "f" | "forbidden" => Ok(Self::Resolve(RuleResolve::Status(403))),
            "g" | "gone" => Ok(Self::Resolve(RuleResolve::Status(410))),
            "" => Ok(Self::Resolve(RuleResolve::Status(parse_status(s, 403)?))),
            _ => Err(RuleError::InvalidFlag(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let rule = Rule::from_str(" ^/replace/[A-Z]+/$ - [I,F]").unwrap();
        assert_eq!(rule.rewrite, "-".to_owned());
        assert_eq!(rule.flags.len(), 2);
        assert!(matches!(
            rule.flags.get(0),
            Some(RuleFlag::Mod(RuleMod::NoCase))
        ));
        assert!(matches!(
            rule.flags.get(1),
            Some(RuleFlag::Resolve(RuleResolve::Status(403)))
        ));
    }

    #[test]
    fn test_simple_replace() {
        let rule = Rule::from_str(r" ^/file/(.*)$ /new/$1 [NE]").unwrap();
        assert_eq!(rule.try_rewrite("/no/match"), None);
        assert_eq!(
            rule.try_rewrite("/file/match"),
            Some("/new/match".to_owned())
        );
        assert_eq!(
            rule.try_rewrite("/file/multiple/match"),
            Some("/new/multiple/match".to_owned())
        );
    }

    #[test]
    fn test_multi_replace() {
        let rule = Rule::from_str(r" ^/file/(\w+)/break/(\w+)$ /new/$2/$1 ").unwrap();
        assert_eq!(rule.try_rewrite("/file/partial/"), None);
        assert_eq!(rule.try_rewrite("/file/partial/break/"), None);
        assert_eq!(rule.try_rewrite("/file/partial/break/test "), None);
        assert_eq!(
            rule.try_rewrite("/file/one/break/two"),
            Some("/new/two/one".to_owned())
        );
    }

    #[test]
    fn test_named_replace() {
        let rule = Rule::from_str(r" ^/file/(?P<name>\w+)$ /$name ").unwrap();
        assert_eq!(rule.try_rewrite("/file/"), None);
        assert_eq!(
            rule.try_rewrite("/file/named_file"),
            Some("/named_file".to_owned())
        );
    }
}
