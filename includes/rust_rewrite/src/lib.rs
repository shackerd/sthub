//! Framework agnostic reimplementation of HTTPD's [mod_rewrite](https://httpd.apache.org/docs/current/mod/mod_rewrite.html).
//!
//! # Example
//!
//! ```
//! use mod_rewrite::Engine;
//!
//! let mut engine = Engine::default();
//! engine.add_rules(r#"
//!   RewriteRule /file/(.*)     /tmp/$1      [L]
//!   RewriteRule /redirect/(.*) /location/$1 [R=302]
//!   RewriteRule /blocked/(.*)  -            [F]
//! "#).expect("failed to process rules");
//!
//! let uri = "http://localhost/file/my/document.txt";
//! let result = engine.rewrite(uri).unwrap();
//! println!("{result:?}");
//! ```
use std::str::FromStr;

mod conditions;
pub mod error;
mod expr;
mod extra;
mod rule;

use conditions::EngineCtx;
use error::{EngineError, ExpressionError};
use expr::ExpressionList;

pub use conditions::{Condition, context};
pub use expr::{ExprGroup, Expression, Rewrite};
pub use extra::State;
pub use rule::Rule;

/// Expression Engine for Proccessing Rewrite Rules
///
/// Supports a subset of [official](https://httpd.apache.org/docs/current/mod/mod_rewrite.html)
/// `mod_rewrite` expressions.
///
/// # Example
///
/// ```
/// use mod_rewrite::Engine;
///
/// let mut engine = Engine::default();
/// engine.add_rules(r#"
///     RewriteRule /file/(.*)     /tmp/$1      [L]
///     RewriteRule /redirect/(.*) /location/$1 [R=302]
///     RewriteRule /blocked/(.*)  -            [F]
/// "#).expect("failed to process rules");
///
/// let uri = "http://localhost/file/my/document.txt";
/// let result = engine.rewrite(uri).unwrap();
/// println!("{result:?}");
/// ```
#[derive(Debug, Default, Clone)]
pub struct Engine {
    groups: Vec<ExprGroup>,
}

impl Engine {
    /// Configure max number of loops over entire ruleset during
    /// rewrite before error
    ///
    /// Default is 10
    pub fn max_iterations(mut self, iterations: usize) -> Self {
        self.groups = self
            .groups
            .into_iter()
            .map(|g| g.max_iterations(iterations))
            .collect();
        self
    }

    /// Parse additonal [`Expression`]s to append as [`ExprGroup`]s to the
    /// existing engine.
    #[inline]
    pub fn add_rules(&mut self, rules: &str) -> Result<&mut Self, ExpressionError> {
        let groups = ExpressionList::from_str(rules)?.groups();
        self.groups.extend(groups);
        Ok(self)
    }

    /// Evaluate the given URI against the configured [`ExprGroup`] instances
    /// defined and generate a [`Rewrite`] response.
    ///
    /// This method skips using [`EngineCtx`] which is used to suppliment
    /// [`Condition`] expressions. If you are NOT making use of `RewriteCond`
    /// rules, this method may be simpler to use.
    ///
    /// See [`Engine::rewrite_ctx`] for more details.
    #[inline]
    pub fn rewrite(&self, uri: &str) -> Result<Rewrite, EngineError> {
        let mut ctx = EngineCtx::default();
        self.rewrite_ctx(uri, &mut ctx)
    }

    /// Evaluate the given URI against the configured [`ExprGroup`] instances
    /// defined and generate a [`Rewrite`] response.
    ///
    /// This method uses an additional [`EngineCtx`] which is used to suppliment
    /// variables expanded in [`Condition`] expressions.
    ///
    /// If your engine is using `RewriteCond` rules, you will want to use this
    /// method with a complete `EngineCtx`. See [`Engine::rewrite`] for a simpler
    /// alternative.
    pub fn rewrite_ctx(&self, uri: &str, ctx: &mut EngineCtx) -> Result<Rewrite, EngineError> {
        let (mut uri, query) = extra::split_query(uri);
        for group in self.groups.iter().filter(|g| g.match_conditions(ctx)) {
            uri = match group.rewrite(&uri)? {
                Rewrite::Uri(uri) => uri,
                status => return Ok(status.with_query(query)),
            };
        }
        Ok(Rewrite::Uri(uri).with_query(query))
    }
}

impl FromStr for Engine {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let groups = ExpressionList::from_str(s)?.groups();
        Ok(Self { groups })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups() {
        let mut engine = Engine::default();
        engine
            .add_rules(
                r#"
            RewriteRule /static/(.*) /files/$1 [NE,L]

            RewriteRule /(.*)        /index?page=$1
        "#,
            )
            .unwrap();

        let r = engine.rewrite("/static/1/2").unwrap();
        assert!(matches!(r, Rewrite::Uri(uri) if uri == "/index?page=files%2F1%2F2"));

        let r = engine.rewrite("/1/2/3?a=b").unwrap();
        println!("{r:?}");
        assert!(matches!(r, Rewrite::Uri(uri) if uri == "/index?page=1%2F2%2F3&a=b"));
    }

    #[test]
    fn test_query() {
        let mut engine = Engine::default();
        engine
            .add_rules(
                r#"
            RewriteRule /static/(.*) /files/$1 [NE,END]

            RewriteRule /(.*)        /index?page=$1
        "#,
            )
            .unwrap();

        let r = engine.rewrite("/static/1/2?a=b").unwrap();
        assert!(matches!(r, Rewrite::EndUri(uri) if uri == "/files/1/2?a=b"));

        let r = engine.rewrite("/1/2/3?a=b").unwrap();
        println!("{r:?}");
        assert!(matches!(r, Rewrite::Uri(uri) if uri == "/index?page=1%2F2%2F3&a=b"));
    }
}
