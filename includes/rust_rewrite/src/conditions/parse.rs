use super::error::CondError;

pub(crate) fn end_quote(s: &str, index: usize, quote: char) -> Result<usize, CondError> {
    let mut backslashes = 0;
    for (i, c) in s.chars().skip(index).enumerate() {
        if c == quote && backslashes % 2 == 0 {
            return Ok(index + i);
        }
        if c == '\\' {
            backslashes += 1;
            continue;
        }
        backslashes = 0;
    }
    Err(CondError::UnclosedQuotation(s[index - 1..].to_owned()))
}

pub(crate) fn tokenize(s: &str) -> Result<Vec<String>, CondError> {
    let mut expressions: Vec<String> = Vec::new();
    let mut expression: Vec<char> = Vec::new();
    let mut index = 0;
    while index < s.len() - 1 {
        for (i, c) in s.chars().enumerate().skip(index) {
            index = i;
            if c.is_whitespace() && expression.is_empty() {
                continue;
            }
            if c.is_whitespace() {
                expressions.push(expression.iter().collect());
                expression.clear();
                continue;
            }
            if c == '\'' || c == '"' {
                index = end_quote(s, i + 1, c)?;
                expressions.push(s[i + 1..index].to_owned());
                index += 1;
                break;
            }
            expression.push(c);
        }
    }
    if !expression.is_empty() {
        expressions.push(expression.iter().collect());
    }
    Ok(expressions)
}

#[inline]
pub(crate) fn matches_start(s: &str, matches: &'static [char]) -> Option<char> {
    matches.iter().find(|c| s.starts_with(**c)).copied()
}
