use super::{Effect, Ident, Integer, Item, QualifiedIdent, Statement, TypedIdent};
use crate::{span::Span, token::TokenKind};

/// A single case in an if-else ladder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conditional {
    pub condition: Expr,
    pub then_body: Vec<Statement>,
}

/// Expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// (Potentially) qualified identifier.
    Ident(QualifiedIdent),
    /// Simple integer literal.
    Int(Integer),
    /// The escape continuation for functions.
    Return,
    /// The implicit continuation (return for closures).
    Continue,
    /// A binary expression of a single operator
    Binary {
        op: Operator,
        operands: Vec<Expr>,
    },
    /// Member access.
    Member {
        recv: Box<Expr>,
        member: Ident,
    },
    /// Function call.
    Call {
        /// The function expression.
        func: Box<Expr>,
        /// The arguments to the function.
        args: Vec<Expr>,
    },
    /// Block-based function call.
    BlockCall {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Closure block, with or without parameters.
    Closure {
        params: Vec<TypedIdent>,
        stmts: Vec<Statement>,
    },
    /// If-else ladder.
    Conditional {
        cases: Vec<Conditional>,
        /// may be empty
        final_else: Vec<Statement>,
    },
    Handler {
        impl_effects: Vec<Effect>,
        items: Vec<Item>,
    },
    Do {
        stmts: Vec<Statement>,
    },
    DoWith {
        stmts: Vec<Statement>,
        handler: Box<Expr>,
    },
    /// Error node.
    Error {
        err_span: Span,
    },
}

impl Expr {
    /// Determines whether this expression ends with a block.
    pub fn is_block_expr(&self) -> bool {
        match self {
            Self::BlockCall { .. }
            | Self::Conditional { .. }
            | Self::Closure { .. }
            | Self::Handler { .. }
            | Self::Do { .. } => true,
            Self::DoWith { handler, .. } => handler.is_block_expr(),
            _ => false,
        }
    }
}

/// Binary operator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnexpectedError;

impl TryFrom<TokenKind> for Operator {
    type Error = UnexpectedError;

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Sub),
            TokenKind::Star => Ok(Self::Mul),
            TokenKind::Slash => Ok(Self::Div),
            TokenKind::Percent => Ok(Self::Rem),
            TokenKind::DoubleEquals => Ok(Self::Eq),
            TokenKind::ExclaimEquals => Ok(Self::NotEq),
            TokenKind::Gt => Ok(Self::Gt),
            TokenKind::GtEquals => Ok(Self::Ge),
            TokenKind::Lt => Ok(Self::Lt),
            TokenKind::LtEquals => Ok(Self::Le),
            _ => Err(UnexpectedError),
        }
    }
}
