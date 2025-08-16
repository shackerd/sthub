use std::str::FromStr;

use super::error::ExpressionError;

#[inline]
pub(crate) fn split_query(uri: &str) -> (String, &str) {
    uri.split_once('?')
        .map(|(b, q)| (b.to_owned(), q))
        .unwrap_or_else(|| (uri.to_owned(), ""))
}

#[inline]
pub(crate) fn join_query(mut uri: String, query: &str) -> String {
    if query.is_empty() {
        return uri;
    }
    match uri.contains('?') {
        true => uri.push('&'),
        false => uri.push('?'),
    }
    uri.push_str(query);
    uri
}

/// Singular `RewriteEngine` expression definition.
///
/// Considered a breakpoint for [`ExprGroup`](super::ExprGroup)
/// and enables/disables the entire group based on the configured
/// state.
#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    On,
    Off,
}

impl FromStr for State {
    type Err = ExpressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(ExpressionError::InvalidStateRule(s.to_owned())),
        }
    }
}
