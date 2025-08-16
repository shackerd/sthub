use std::str::FromStr;

use crate::extra;

use super::conditions::{Condition, EngineCtx};
use super::error::{EngineError, ExpressionError};
use super::extra::State;
use super::rule::{Rule, RuleResolve, RuleShift};

/// Rewrite result.
///
/// Includes either the re-write uri, or the instant http-response.
#[derive(Debug)]
pub enum Rewrite {
    Uri(String),
    EndUri(String),
    Redirect(String, u16),
    StatusCode(u16),
}

impl Rewrite {
    /// Pass query-string back into uri after rewrite evaluation
    pub(crate) fn with_query(self, query: &str) -> Self {
        match self {
            Self::Uri(uri) => Self::Uri(extra::join_query(uri, query)),
            Self::EndUri(uri) => Self::EndUri(extra::join_query(uri, query)),
            Self::Redirect(uri, sc) => Self::Redirect(extra::join_query(uri, query), sc),
            Self::StatusCode(sc) => Self::StatusCode(sc),
        }
    }
}

/// Logical grouping of [`Expression`] instances.
///
/// Associates a list [`Condition`] instances that guard
/// rewrites defined by [`Rule`].
#[derive(Debug, Clone)]
pub struct ExprGroup {
    conditions: Vec<Condition>,
    rules: Vec<Rule>,
    enabled: bool,
    max_iterations: usize,
}

impl ExprGroup {
    /// Build a new [`ExprGroup`] instance from the specified
    /// list of [`Expression`] instances.
    ///
    /// This should contains all rules related to one another
    /// with [`Condition`] instances leading into [`Rule`] instances after.
    pub fn new(expressions: Vec<Expression>) -> Self {
        let mut conditions = Vec::new();
        let mut rules = Vec::new();
        let mut enabled = true;
        for expr in expressions {
            match expr {
                Expression::Condition(cond) => conditions.push(cond),
                Expression::Rule(rule) => rules.push(rule),
                Expression::State(state) => enabled = matches!(state, State::On),
            }
        }
        Self {
            conditions,
            rules,
            enabled,
            max_iterations: 10,
        }
    }

    /// Configure max number of loops over entire ruleset during
    /// rewrite before error
    ///
    /// Default is 10
    pub fn max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }

    /// Check all relevant [`Condition`] expressions are met.
    ///
    /// This method guards [`ExprGroup::rewrite`].
    pub fn match_conditions(&self, ctx: &mut EngineCtx) -> bool {
        if !self.enabled {
            return false;
        }
        let (or, and): (Vec<_>, Vec<_>) = self.conditions.iter().partition(|c| c.is_or());
        or.into_iter().any(|c| c.is_met(ctx)) || and.into_iter().all(|c| c.is_met(ctx))
    }

    /// Evaluate the given URI against the configured [`Rule`] definitions
    /// and generate a [`Rewrite`] response.
    pub fn rewrite(&self, uri: &str) -> Result<Rewrite, EngineError> {
        let mut next_index = 0;
        let mut iterations = 0;

        let (mut uri, query) = extra::split_query(uri);
        while iterations < self.max_iterations {
            iterations += 1;
            let Some((index, rule, new_uri)) = self
                .rules
                .iter()
                .enumerate()
                .skip(next_index)
                .find_map(|(i, r)| Some((i, r, r.try_rewrite(&uri)?)))
            else {
                break;
            };

            uri = new_uri;
            next_index = index + 1;
            if let Some(shift) = rule.shift() {
                match shift {
                    RuleShift::Next => next_index = 0,
                    RuleShift::Last => break,
                    RuleShift::End => return Ok(Rewrite::EndUri(uri).with_query(query)),
                    RuleShift::Skip(shift) => next_index += *shift as usize,
                }
                continue;
            }
            if let Some(resolve) = rule.resolve() {
                match resolve {
                    RuleResolve::Status(status) => return Ok(Rewrite::StatusCode(*status)),
                    RuleResolve::Redirect(status) => {
                        return Ok(Rewrite::Redirect(uri, *status).with_query(query));
                    }
                }
            }
        }

        match iterations >= self.max_iterations {
            true => Err(EngineError::TooManyIterations),
            false => Ok(Rewrite::Uri(uri).with_query(query)),
        }
    }
}

/// Categorization and deserializion for [`ExprGroup`] instances
/// made from a list of flat [`Expression`] instances.
///
/// Separates [`Expression`] instances into groups by their
/// association to previous [`Condition`] rules and whitespace.
#[derive(Debug)]
pub(crate) struct ExpressionList(pub Vec<Vec<Expression>>);

impl ExpressionList {
    /// Convert [`ExpressionList`] into Vec of [`ExprGroup`]
    #[inline]
    pub fn groups(self) -> Vec<ExprGroup> {
        self.0.into_iter().map(ExprGroup::new).collect()
    }
}

impl FromStr for ExpressionList {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut list = Vec::new();
        let mut group: Vec<Expression> = Vec::new();
        for line in s
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.starts_with("//"))
        {
            if line.is_empty() {
                list.push(group.clone());
                group.clear();
                continue;
            }
            let expr = Expression::from_str(line)?;
            if matches!(expr, Expression::State(_))
                || (matches!(expr, Expression::Condition(_))
                    && group
                        .last()
                        .is_some_and(|e| matches!(e, Expression::Rule(_))))
            {
                list.push(group.clone());
                group.clear();
            }
            group.push(expr);
        }
        if !group.is_empty() {
            list.push(group);
        }
        Ok(Self(list.into_iter().filter(|g| !g.is_empty()).collect()))
    }
}

/// All possible expression types allowed within `mod_rewrite`
///
/// Will eventually support RewriteEngine/RewriteCond/RewriteRule/RewriteBase
#[derive(Clone, Debug)]
pub enum Expression {
    Condition(Condition),
    Rule(Rule),
    State(State),
}

impl FromStr for Expression {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ident, expr) = s
            .split_once(char::is_whitespace)
            .ok_or(ExpressionError::MissingIdentifier)?;
        match ident.to_lowercase().as_str() {
            "rule" | "rewrite" | "rewriterule" => Ok(Self::Rule(Rule::from_str(expr)?)),
            "cond" | "condition" | "rewritecond" => Ok(Self::Condition(Condition::from_str(expr)?)),
            "state" | "engine" | "rewriteengine" => Ok(Self::State(State::from_str(expr)?)),
            _ => Err(ExpressionError::InvalidIdentifier(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups() {
        let groups = ExpressionList::from_str(
            r#"
            RewriteCond /var/www/%{REQUEST_URI} !-f
            RewriteRule ^/file/(.*)$ /file2/$1  [R=303]

            RewriteRule /rewrite/[A-Z]+ /redirect/$1 [NC,R]
            RewriteCond ${SERVER_PORT} -eq 4000
            RewriteRule /(.*) /index.php?path=$1
            RewriteEngine off
            RewriteRule / - [F]
        "#,
        )
        .unwrap()
        .groups();

        assert_eq!(groups.len(), 4);
        assert_eq!(groups[0].conditions.len(), 1);
        assert_eq!(groups[0].rules.len(), 1);
        assert_eq!(groups[0].enabled, true);
        assert_eq!(groups[1].conditions.len(), 0);
        assert_eq!(groups[1].rules.len(), 1);
        assert_eq!(groups[1].enabled, true);
        assert_eq!(groups[2].conditions.len(), 1);
        assert_eq!(groups[2].rules.len(), 1);
        assert_eq!(groups[2].enabled, true);
        assert_eq!(groups[3].conditions.len(), 0);
        assert_eq!(groups[3].rules.len(), 1);
        assert_eq!(groups[3].enabled, false);
    }

    #[test]
    fn test_rules() {
        let groups = ExpressionList::from_str(
            r#"
            RewriteRule /skip      /new/test      [S=2]
            RewriteRule /skip      -              [F]
            RewriteRule /new/(.*)  /index?page=$1 [R=303]
            RewriteRule /new/(.*)  -              [G]
            RewriteRule /(.*)      /new/$1        [N,NE]
        "#,
        )
        .unwrap()
        .groups();

        assert_eq!(groups.len(), 1);
        let group = &groups[0];

        let r = group.rewrite("/skip").unwrap();
        assert!(matches!(r, Rewrite::StatusCode(code) if code == 410));

        let r = group.rewrite("/hello/world").unwrap();
        assert!(
            matches!(r, Rewrite::Redirect(uri, sc) if uri == "/index?page=hello%2Fworld" && sc == 303)
        );
    }

    #[test]
    fn test_query() {
        let groups = ExpressionList::from_str(
            r#"
            RewriteRule /static/(.*) /files/$1 [NE,L]
            RewriteRule /(.*)        /index?page=$1
        "#,
        )
        .unwrap()
        .groups();

        assert_eq!(groups.len(), 1);
        let group = &groups[0];

        let r = group.rewrite("/static/1/2?a=b").unwrap();
        assert!(matches!(r, Rewrite::Uri(uri) if uri == "/files/1/2?a=b"));

        let r = group.rewrite("/1/2/3?a=b").unwrap();
        assert!(matches!(r, Rewrite::Uri(uri) if uri == "/index?page=1%2F2%2F3&a=b"));
    }

    #[test]
    fn test_overflow() {
        let groups = ExpressionList::from_str(
            r#"
            RewriteRule /skip/forbidden -       [F]
            RewriteRule /skip/gone      -       [G]
            RewriteRule /(.*)           /new/$1 [N]
        "#,
        )
        .unwrap()
        .groups();

        assert_eq!(groups.len(), 1);
        let group = &groups[0];

        let r = group.rewrite("/skip");
        assert!(matches!(r, Err(EngineError::TooManyIterations)));
    }
}
