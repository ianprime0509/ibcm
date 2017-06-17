//! The abstract syntax tree produced by the parser.

use ibcmc::lexer::{Ident, Literal};

/// Represents a single block (e.g. the definition of a function, or a block delimited by `{}`).
/// A block is merely a vector of statements.
#[derive(Clone,Debug)]
pub struct Block(pub Vec<Stmt>);

/// Represents a single statement (e.g. a line ending in a semicolon).
#[derive(Clone,Debug)]
pub enum Stmt {
    /// A block (e.g. delimited by `{}`).
    Block(Block),
    /// An assignment (e.g. `i = 3`).
    Assign(Ident, Expr),
    /// A compound assignment (e.g. `i += 3`).
    CompoundAssign(Ident, BinOp, Expr),
    /// A declaration (e.g. `int i`).
    ///
    /// The first member specifies whether a constant is being declared.
    Decl(bool, Type, Ident),
    /// An initialization (e.g. `int i = 2`).
    Init(bool, Type, Ident, Expr),
    /// An expression.
    Expr(Expr),
    /// The empty statement.
    Empty,
}

/// Represents a single expression (e.g. `i + 3`).
#[derive(Clone,Debug)]
pub enum Expr {
    /// A binary operation (e.g. `i + 3`).
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// An identifier.
    Ident(Ident),
    /// A literal.
    Literal(Literal),
}

/// An enumeration of all possible types.
#[derive(Clone,Debug)]
pub enum Type {
    /// An integer (`u16`).
    Int,
}

/// All the binary operations which can be performed on a variable.
#[derive(Clone,Debug)]
pub enum BinOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
}

