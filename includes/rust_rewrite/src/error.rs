use std::num::ParseIntError;

use thiserror::Error;

pub use super::conditions::CondError;

/// Errors when running expression engine
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Too many iterations on rule processing. Infintite loop")]
    TooManyIterations,
}

/// Errors when parsing all rewrite expressions
#[derive(Debug, Error)]
pub enum ExpressionError {
    #[error("Missing expression identifier")]
    MissingIdentifier,

    #[error("Invalid rule identifier")]
    InvalidIdentifier(String),

    #[error("Invalid state rule")]
    InvalidStateRule(String),

    #[error("Error when parisng condition rule")]
    ConditionError(#[from] CondError),

    #[error("Error when parisng rewrite rule")]
    RuleError(#[from] RuleError),
}

/// Errors when parsing rewrite rules
#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Rule is missing a pattern")]
    MissingPattern,

    #[error("Invalid regex in rule rewrite pattern")]
    InvalidRegex(String),

    #[error("Rule is missing a rewrite expression")]
    MissingRewrite,

    #[error("Invalid suffix to rule expression")]
    InvalidSuffix(String),

    #[error("Rule flag definitions missing brackets")]
    FlagsMissingBrackets(String),

    #[error("Rule flags empty")]
    FlagsEmpty,

    #[error("Rule flags used are mutually exclusive")]
    FlagsMutuallyExclusive,

    #[error("Invalid flag in rule definition")]
    InvalidFlag(String),

    #[error("Invalid number in rule definition")]
    InvalidFlagNumber(#[from] ParseIntError),

    #[error("Invalid status code in rule definition")]
    InvalidFlagStatus(String),
}
