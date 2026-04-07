//! Tokenizer for the UOR EBNF surface syntax.

/// A token in the UOR term language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Identifier (type name, variable name, operation name).
    Ident(String),
    /// Integer literal.
    IntLit(u64),
    /// `@` for quantum level annotations.
    At,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `:`
    Colon,
    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `=`
    Eq,
    /// `=>`
    FatArrow,
    /// `true` or `false`
    BoolLit(bool),
    /// String literal (`"..."`)
    StringLit(String),
    /// End of input.
    Eof,
}

/// Tokenizes input into a sequence of tokens.
///
/// # Errors
///
/// Returns an error string if an unexpected character is encountered.
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '@' => {
                tokens.push(Token::At);
                i += 1;
            }
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            ';' => {
                tokens.push(Token::Semi);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::FatArrow);
                    i += 2;
                } else {
                    tokens.push(Token::Eq);
                    i += 1;
                }
            }
            '"' => {
                i += 1; // skip opening quote
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                        match chars[i] {
                            '"' => s.push('"'),
                            '\\' => s.push('\\'),
                            'n' => s.push('\n'),
                            'r' => s.push('\r'),
                            't' => s.push('\t'),
                            other => {
                                s.push('\\');
                                s.push(other);
                            }
                        }
                    } else {
                        s.push(chars[i]);
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("Unterminated string literal".to_string());
                }
                i += 1; // skip closing quote
                tokens.push(Token::StringLit(s));
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let value = num_str
                    .parse::<u64>()
                    .map_err(|e| format!("Invalid integer literal: {e}"))?;
                tokens.push(Token::IntLit(value));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "true" => tokens.push(Token::BoolLit(true)),
                    "false" => tokens.push(Token::BoolLit(false)),
                    _ => tokens.push(Token::Ident(word)),
                }
            }
            c => return Err(format!("Unexpected character: '{c}'")),
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
