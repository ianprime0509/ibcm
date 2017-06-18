//! The abstract syntax tree produced by the parser.

use ibcmc::lexer::{Ident, Literal};

/// Represents a single block (e.g. the definition of a function, or a block delimited by `{}`).
/// A block is merely a vector of statements.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Block(pub Vec<Stmt>);

/// Represents a single statement (e.g. a line ending in a semicolon, or even another block).
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Stmt {
    /// A function declaration.
    ///
    /// The members are: declaration (return type and name), parameters, and the body.
    Function(Decl, Vec<Decl>, Block),
    /// A block (e.g. delimited by `{}`).
    Block(Block),
    /// An assignment (e.g. `i = 3`).
    Assign(Ident, Expr),
    /// A compound assignment (e.g. `i += 3`).
    CompoundAssign(Ident, BinOp, Expr),
    /// A declaration (e.g. `int i`).
    ///
    /// The first member specifies whether a constant is being declared.
    Decl(Decl),
    /// An initialization (e.g. `int i = 2`).
    Init(Decl, Expr),
    /// An expression.
    Expr(Expr),
    /// The empty statement.
    Empty,
}

/// Represents a single expression (e.g. `i + 3`).
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Expr {
    /// A binary operation (e.g. `i + 3`).
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// An identifier.
    Ident(Ident),
    /// A literal.
    Literal(Literal),
}

/// Represents a variable declaration or function parameter.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Decl {
    /// Whether the variable is constant.
    pub is_const: bool,
    /// The type.
    pub ty: Type,
    /// The name.
    pub name: Ident,
}

/// An enumeration of all possible types.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Type {
    /// An integer (`u16`).
    Int,
}

/// All the binary operations which can be performed on a variable.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum BinOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
}

