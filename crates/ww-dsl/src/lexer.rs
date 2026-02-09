use logos::Logos;
use std::fmt;

/// Token type for the Weltenwanderer DSL.
///
/// The lexer is deliberately simple — all keyword recognition happens in the parser.
/// Words like "member", "of", "located", "at" are all `Token::Word`. The parser
/// combines them into multi-word keywords based on grammatical context.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Left brace `{`.
    LBrace,
    /// Right brace `}`.
    RBrace,
    /// Left bracket `[`.
    LBracket,
    /// Right bracket `]`.
    RBracket,
    /// Left parenthesis `(`.
    LParen,
    /// Right parenthesis `)`.
    RParen,
    /// Comma separator `,`.
    Comma,
    /// Newline character (statement separator).
    Newline,
    /// Triple-quoted doc string (`"""..."""`).
    DocString(String),
    /// Double-quoted string literal.
    Str(String),
    /// Integer literal (supports Rust-style underscores and negatives).
    /// Stores both the parsed value and the original source text to preserve
    /// leading zeros when numbers appear in entity names (e.g. `022`).
    Integer(i64, String),
    /// Floating-point literal. Stores parsed value and original source text.
    Float(f64, String),
    /// Bare word (identifier or keyword, disambiguated by the parser).
    Word(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Newline => write!(f, "newline"),
            Token::DocString(_) => write!(f, "doc string"),
            Token::Str(s) => write!(f, "\"{s}\""),
            Token::Integer(_, s) => write!(f, "{s}"),
            Token::Float(_, s) => write!(f, "{s}"),
            Token::Word(w) => write!(f, "{w}"),
        }
    }
}

/// Internal logos token — borrows from source to avoid allocations during lexing.
/// Converted to owned `Token` after lexing.
#[derive(Logos, Debug)]
#[logos(skip r"[ \t\r]+")]
#[logos(skip r"--[^\n]*")]
enum RawToken {
    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token("\n")]
    Newline,

    #[token("\"\"\"")]
    DocStringStart,

    #[regex(r#""[^"\n]*""#)]
    Str,

    #[regex(r"-?[0-9][0-9_]*\.[0-9][0-9_]*")]
    Float,

    #[regex(r"-?[0-9][0-9_]*")]
    Integer,

    #[regex(r"[a-zA-Z][a-zA-Z0-9_'-]*")]
    Word,
}

/// A lexer error with source location.
#[derive(Debug, Clone)]
pub struct LexError {
    /// Byte range of the erroneous input in the source.
    pub span: std::ops::Range<usize>,
    /// Human-readable description of the lexer error.
    pub message: String,
}

/// Lex source code into a sequence of `(Token, Span)` pairs.
///
/// Returns the token stream and any lexer errors. Lexing continues past errors
/// to collect as many tokens as possible (important for IDE/LSP support).
pub fn lex(source: &str) -> (Vec<(Token, std::ops::Range<usize>)>, Vec<LexError>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lexer = RawToken::lexer(source);

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(raw) => {
                let token = match raw {
                    RawToken::LBrace => Token::LBrace,
                    RawToken::RBrace => Token::RBrace,
                    RawToken::LBracket => Token::LBracket,
                    RawToken::RBracket => Token::RBracket,
                    RawToken::LParen => Token::LParen,
                    RawToken::RParen => Token::RParen,
                    RawToken::Comma => Token::Comma,
                    RawToken::Newline => Token::Newline,
                    RawToken::DocStringStart => {
                        // Scan forward for closing """
                        let remainder = lexer.remainder();
                        match remainder.find("\"\"\"") {
                            Some(end_idx) => {
                                let content = &remainder[..end_idx];
                                lexer.bump(end_idx + 3);
                                let full_span = span.start..lexer.span().start;
                                tokens.push((
                                    Token::DocString(content.trim().to_string()),
                                    full_span,
                                ));
                                continue;
                            }
                            None => {
                                errors.push(LexError {
                                    span: span.clone(),
                                    message: "unterminated doc string (missing closing \"\"\")"
                                        .to_string(),
                                });
                                continue;
                            }
                        }
                    }
                    RawToken::Str => {
                        let slice = lexer.slice();
                        Token::Str(unescape(&slice[1..slice.len() - 1]))
                    }
                    RawToken::Float => {
                        let raw = lexer.slice().to_string();
                        match raw.replace('_', "").parse::<f64>() {
                            Ok(n) => Token::Float(n, raw),
                            Err(_) => {
                                errors.push(LexError {
                                    span: span.clone(),
                                    message: format!("invalid float literal: {raw}"),
                                });
                                continue;
                            }
                        }
                    }
                    RawToken::Integer => {
                        let raw = lexer.slice().to_string();
                        match raw.replace('_', "").parse::<i64>() {
                            Ok(n) => Token::Integer(n, raw),
                            Err(_) => {
                                errors.push(LexError {
                                    span: span.clone(),
                                    message: format!("invalid integer literal: {raw}"),
                                });
                                continue;
                            }
                        }
                    }
                    RawToken::Word => Token::Word(lexer.slice().to_string()),
                };
                tokens.push((token, span));
            }
            Err(()) => {
                errors.push(LexError {
                    span: span.clone(),
                    message: format!("unexpected character: {:?}", &source[span.clone()]),
                });
            }
        }
    }

    (tokens, errors)
}

/// Process escape sequences in a string literal.
///
/// Supports `\\`, `\n`, `\t`, `\"`. Unknown sequences are kept as-is.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_world_declaration() {
        let source = r#"world "The Iron Kingdoms" {
    genre "high fantasy"
}"#;
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty(), "errors: {errors:?}");

        let types: Vec<_> = tokens.iter().map(|(t, _)| format!("{t}")).collect();
        assert_eq!(types[0], "world");
        assert_eq!(types[1], "\"The Iron Kingdoms\"");
        assert_eq!(types[2], "{");
    }

    #[test]
    fn lex_entity_declaration() {
        let source = "Kael Stormborn is a character {\n    species human\n}";
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty(), "errors: {errors:?}");

        let words: Vec<_> = tokens
            .iter()
            .filter_map(|(t, _)| match t {
                Token::Word(w) => Some(w.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            words,
            vec![
                "Kael",
                "Stormborn",
                "is",
                "a",
                "character",
                "species",
                "human"
            ]
        );
    }

    #[test]
    fn lex_integers_with_underscores() {
        let (tokens, errors) = lex("population 45_000");
        assert!(errors.is_empty());

        assert!(matches!(&tokens[1].0, Token::Integer(45_000, _)));
    }

    #[test]
    fn lex_negative_integers() {
        let (tokens, errors) = lex("year -1247");
        assert!(errors.is_empty());

        assert!(matches!(&tokens[1].0, Token::Integer(-1247, _)));
    }

    #[test]
    fn lex_doc_string() {
        let source = "\"\"\"\nHello world.\nSecond line.\n\"\"\"";
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty(), "errors: {errors:?}");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::DocString(s) if s == "Hello world.\nSecond line."));
    }

    #[test]
    fn lex_list() {
        let (tokens, errors) = lex("[brave, stubborn, loyal]");
        assert!(errors.is_empty());

        let types: Vec<_> = tokens.iter().map(|(t, _)| format!("{t}")).collect();
        assert_eq!(
            types,
            vec!["[", "brave", ",", "stubborn", ",", "loyal", "]"]
        );
    }

    #[test]
    fn lex_comments_are_skipped() {
        let source = "-- This is a comment\nworld";
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty());

        // The comment is skipped, only newline + "world" remain
        let non_newline: Vec<_> = tokens
            .iter()
            .filter(|(t, _)| !matches!(t, Token::Newline))
            .collect();
        assert_eq!(non_newline.len(), 1);
        assert!(matches!(&non_newline[0].0, Token::Word(w) if w == "world"));
    }

    #[test]
    fn lex_float() {
        let (tokens, errors) = lex("3.14");
        assert!(errors.is_empty());
        #[allow(clippy::approx_constant)]
        let expected = 3.14;
        assert!(matches!(&tokens[0].0, Token::Float(f, _) if (*f - expected).abs() < f64::EPSILON));
    }

    #[test]
    fn lex_preserves_spans() {
        let source = "hello world";
        let (tokens, _) = lex(source);
        assert_eq!(tokens[0].1, 0..5);
        assert_eq!(tokens[1].1, 6..11);
    }

    #[test]
    fn unescape_newlines() {
        assert_eq!(unescape(r"hello\nworld"), "hello\nworld");
    }

    #[test]
    fn unescape_tabs() {
        assert_eq!(unescape(r"col1\tcol2"), "col1\tcol2");
    }

    #[test]
    fn unescape_backslash() {
        assert_eq!(unescape(r"path\\file"), "path\\file");
    }

    #[test]
    fn unescape_quote() {
        assert_eq!(unescape(r#"say \"hello\""#), "say \"hello\"");
    }

    #[test]
    fn unescape_unknown_kept() {
        assert_eq!(unescape(r"\x"), "\\x");
    }

    #[test]
    fn unescape_trailing_backslash() {
        assert_eq!(unescape("trail\\"), "trail\\");
    }

    #[test]
    fn lex_string_with_escapes() {
        let source = r#""line1\nline2\ttab""#;
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty());
        assert!(matches!(&tokens[0].0, Token::Str(s) if s == "line1\nline2\ttab"));
    }
}
