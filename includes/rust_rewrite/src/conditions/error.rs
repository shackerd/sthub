use thiserror::Error;

/// Error when parsing rule condition expression
#[derive(Debug, Error, PartialEq)]
pub enum CondError {
    #[error("Invalid string pattern expression")]
    InvalidPattern(String),

    #[error("Invalid comparison expression")]
    InvalidComparison(String),

    #[error("Invalid filetest expression")]
    InvalidFileTest(String),

    #[error("Quotation never closed in expression")]
    UnclosedQuotation(String),

    #[error("Rule condition expression is empty")]
    EmptyExpression,

    #[error("Rule conditiion is missing comparison")]
    MissingComparison,

    #[error("Invalid expression suffix")]
    InvalidSuffix(String),

    #[error("Missing suffix for comparison")]
    MissingSuffix,

    #[error("Condition flags missing brackets")]
    FlagsMissingBrackets(String),

    #[error("Condition flags are empty")]
    FlagsEmpty,

    #[error("Invalid condition flag")]
    InvalidFlag(String),
}
