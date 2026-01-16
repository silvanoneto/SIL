//! Lexer for LIS language
//!
//! Tokenizes LIS source code into a stream of tokens.

use logos::Logos;
use crate::error::{Error, Result};

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("transform")]
    Transform,
    #[token("type")]
    Type,
    #[token("let")]
    Let,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("loop")]
    Loop,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("feedback")]
    Feedback,
    #[token("emerge")]
    Emerge,

    // Module system keywords
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("mod")]
    Mod,
    #[token("as")]
    As,
    #[token("extern")]
    Extern,

    // Types
    #[token("ByteSil")]
    TByteSil,
    #[token("State")]
    TState,

    // Hardware hints
    #[token("@cpu")]
    HintCpu,
    #[token("@gpu")]
    HintGpu,
    #[token("@npu")]
    HintNpu,
    #[token("@simd")]
    HintSimd,
    #[token("@photonic")]
    HintPhotonic,

    // Layer references (L0-LF)
    #[regex(r"L[0-9A-F]", |lex| {
        let s = lex.slice();
        u8::from_str_radix(&s[1..], 16).ok()
    })]
    Layer(u8),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Literals
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i64),

    #[regex(r"-?[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("**")]
    StarStar,

    #[token("==")]
    EqEq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,

    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,

    #[token("^")]
    Caret,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("~")]
    Tilde,

    #[token("|>")]
    PipeRight,

    // Delimiters
    #[token("=")]
    Eq,
    #[token(";")]
    Semi,
    #[token("::")]
    ColonColon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token("->")]
    Arrow,
}

/// Source location span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self { start, end, line, col }
    }

    pub fn dummy() -> Self {
        Self { start: 0, end: 0, line: 0, col: 0 }
    }

    /// Merge two spans into one that covers both
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            col: if self.line <= other.line { self.col } else { other.col },
        }
    }
}

/// Token with source location
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

pub struct Lexer {
    source: String,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    /// Tokenize returning tokens without spans (backwards compatible)
    pub fn tokenize(&self) -> Result<Vec<Token>> {
        Ok(self.tokenize_with_spans()?.into_iter().map(|st| st.token).collect())
    }

    /// Tokenize returning tokens with source spans
    pub fn tokenize_with_spans(&self) -> Result<Vec<SpannedToken>> {
        let mut tokens = Vec::new();
        let mut lex = Token::lexer(&self.source);

        // Pre-compute line starts for fast line/col lookup
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(self.source.match_indices('\n').map(|(i, _)| i + 1))
            .collect();

        while let Some(token) = lex.next() {
            let byte_span = lex.span();
            let (line, col) = self.offset_to_line_col(&line_starts, byte_span.start);

            match token {
                Ok(tok) => {
                    tokens.push(SpannedToken {
                        token: tok,
                        span: Span::new(byte_span.start, byte_span.end, line, col),
                    });
                }
                Err(_) => {
                    return Err(Error::LexError {
                        message: format!("Unexpected character: '{}'", &self.source[byte_span]),
                        line,
                        col,
                    });
                }
            }
        }

        Ok(tokens)
    }

    /// Convert byte offset to line and column (1-indexed)
    fn offset_to_line_col(&self, line_starts: &[usize], offset: usize) -> (usize, usize) {
        let line = line_starts.partition_point(|&start| start <= offset);
        let line_start = line_starts.get(line.saturating_sub(1)).copied().unwrap_or(0);
        let col = offset - line_start + 1;
        (line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "fn transform let return if else loop use pub mod as extern";
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Fn,
                Token::Transform,
                Token::Let,
                Token::Return,
                Token::If,
                Token::Else,
                Token::Loop,
                Token::Use,
                Token::Pub,
                Token::Mod,
                Token::As,
                Token::Extern,
            ]
        );
    }

    #[test]
    fn test_path_separator() {
        let source = "foo::bar::baz";
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("foo".to_string()),
                Token::ColonColon,
                Token::Ident("bar".to_string()),
                Token::ColonColon,
                Token::Ident("baz".to_string()),
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let source = "main foo_bar x123";
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("main".to_string()),
                Token::Ident("foo_bar".to_string()),
                Token::Ident("x123".to_string()),
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let source = "42 -17 3.14 -2.5e-3";
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Int(42),
                Token::Int(-17),
                Token::Float(3.14),
                Token::Float(-2.5e-3),
            ]
        );
    }

    #[test]
    fn test_layers() {
        let source = "L0 L5 LA LF";
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Layer(0),
                Token::Layer(5),
                Token::Layer(10),
                Token::Layer(15),
            ]
        );
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / ** == != < > |> ^";
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::StarStar,
                Token::EqEq,
                Token::Ne,
                Token::Lt,
                Token::Gt,
                Token::PipeRight,
                Token::Caret,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let source = r#"
            // Line comment
            fn main() {
                /* Block comment */
                let x = 42; /* inline */
            }
        "#;
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert!(tokens.contains(&Token::Fn));
        assert!(tokens.contains(&Token::Ident("main".to_string())));
        assert!(!tokens.iter().any(|t| matches!(t, Token::Ident(s) if s.contains("comment"))));
    }

    #[test]
    fn test_string_literals() {
        let source = r#""hello" "world with spaces""#;
        let tokens = Lexer::new(source).tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::String("hello".to_string()),
                Token::String("world with spaces".to_string()),
            ]
        );
    }
}
