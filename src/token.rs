use std::fmt::{self, Display, Formatter};

use crate::span::Spanned;

/// Type of tokens.
pub type Token = Spanned<TokenKind>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    // Meta
    Eof,
    Unrecognized,
    // Punctuation
    Arrow,
    Colon,
    Comma,
    CurlyL,
    CurlyR,
    Dot,
    DoubleEquals,
    Equals,
    ExclaimEquals,
    Gt,
    GtEquals,
    Lt,
    LtEquals,
    Minus,
    Percent,
    Pipe,
    Plus,
    RoundL,
    RoundR,
    Scope,
    Semi,
    Slash,
    SquareL,
    SquareR,
    Star,
    // Keywords
    Do,
    Effect,
    Else,
    Finally,
    Fn,
    Handle,
    If,
    Import,
    CC,
    Let,
    Recur,
    Resume,
    Return,
    With,
    // Data-carrying
    Number,
    BasePrefixNumber,
    Ident,
}

impl TokenKind {
    pub const KEYWORDS: &'static [Self] = &[
        Self::Do,
        Self::Effect,
        Self::Else,
        Self::Finally,
        Self::Fn,
        Self::Handle,
        Self::If,
        Self::Import,
        Self::CC,
        Self::Let,
        Self::Recur,
        Self::Resume,
        Self::Return,
        Self::With,
    ];

    pub const WIDTH_TWO_PUNCT: &'static [Self] = &[
        Self::Arrow,
        Self::DoubleEquals,
        Self::ExclaimEquals,
        Self::GtEquals,
        Self::LtEquals,
        Self::Scope,
    ];

    pub const WIDTH_ONE_PUNCT: &'static [Self] = &[
        Self::Colon,
        Self::Comma,
        Self::CurlyL,
        Self::CurlyR,
        Self::Equals,
        Self::Gt,
        Self::Lt,
        Self::Dot,
        Self::Minus,
        Self::Percent,
        Self::Pipe,
        Self::Plus,
        Self::RoundL,
        Self::RoundR,
        Self::Semi,
        Self::Slash,
        Self::SquareL,
        Self::SquareR,
        Self::Star,
    ];

    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Arrow => "->",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::CurlyL => "{",
            Self::CurlyR => "}",
            Self::DoubleEquals => "==",
            Self::Equals => "=",
            Self::ExclaimEquals => "!=",
            Self::Gt => ">",
            Self::GtEquals => ">=",
            Self::Lt => "<",
            Self::LtEquals => "<=",
            Self::Dot => ".",
            Self::Minus => "-",
            Self::Percent => "%",
            Self::Pipe => "|",
            Self::Plus => "+",
            Self::RoundL => "(",
            Self::RoundR => ")",
            Self::Scope => "::",
            Self::Semi => ";",
            Self::Slash => "/",
            Self::SquareL => "[",
            Self::SquareR => "]",
            Self::Star => "*",
            Self::Do => "do",
            Self::Effect => "effect",
            Self::Else => "else",
            Self::Finally => "finally",
            Self::Fn => "fn",
            Self::Handle => "handle",
            Self::If => "if",
            Self::Import => "import",
            Self::CC => "k",
            Self::Let => "let",
            Self::Recur => "recur",
            Self::Resume => "resume",
            Self::Return => "return",
            Self::With => "with",
            Self::Ident => "<ident>",
            Self::Number => "<number>",
            Self::BasePrefixNumber => "0Z<number>",
            Self::Eof => "<EOF>",
            Self::Unrecognized => "<?>",
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
