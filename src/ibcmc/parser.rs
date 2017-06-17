//! The IBCMC parser.

use std::io::Result as IoResult;

use ibcmc::errors::*;
use ibcmc::ast::{Type, BinOp, Block, Stmt, Expr};
use ibcmc::lexer::{Lexer, Token, Ident, Keyword};

macro_rules! eparse {
    ($self:ident, $($arg:tt)*) => {
        ErrorKind::Parser(format!($($arg)*), $self.lexer.line()).into()
    }
}

/// Represents the state of a parser.
pub struct Parser<I>
    where I: Iterator<Item = IoResult<u8>>
{
    /// The underlying token stream.
    lexer: Lexer<I>
}

impl<I> Parser<I>
    where I: Iterator<Item = IoResult<u8>>
{
    /// Parses a complete program from the given lexer.
    pub fn parse_from_lexer(lexer: Lexer<I>) -> Result<Block>
    {
        let mut parser = Parser { lexer };
        parser.block()
    }

    /// Parses a block.
    fn block(&mut self) -> Result<Block> {
        let mut stmts = Vec::new();

        while let Some(tok) = self.lexer.next() {
            self.lexer.put_back(tok?);
            stmts.push(self.stmt()?);
        }

        Ok(Block(stmts))
    }

    /// Parses a statement.
    ///
    /// The semicolon at the end of the line will be parsed as a part of this node (or a helper function, like `decl_or_init_stmt`), if relevant.
    fn stmt(&mut self) -> Result<Stmt> {
        Ok(match self.lexer.next().unwrap()? {
            Token::Semi => Stmt::Empty,
            Token::Keyword(Keyword::Const) => match self.decl_or_init_stmt()? {
                Stmt::Decl(_, ty, ident) => Stmt::Decl(true, ty, ident),
                Stmt::Init(_, ty, ident, expr) => Stmt::Init(true, ty, ident, expr),
                _ => unreachable!("error in decl_or_init_stmt()")
            },
            tok @ Token::Keyword(Keyword::Int) => {
                self.lexer.put_back(tok);
                self.decl_or_init_stmt()?
            }
            Token::Ident(ident) => {
                if let Some(tok) = self.lexer.next() {
                    match tok? {
                        Token::Assign => {
                            let expr = self.expr()?;
                            self.expect(Token::Semi)?;
                            Stmt::Assign(ident, expr)
                        }
                        Token::AddAssign => {
                            let expr = self.expr()?;
                            self.expect(Token::Semi)?;
                            Stmt::CompoundAssign(ident, BinOp::Add, expr)
                        }
                        Token::SubAssign => {
                            let expr = self.expr()?;
                            self.expect(Token::Semi)?;
                            Stmt::CompoundAssign(ident, BinOp::Sub, expr)
                        }
                        tok => {
                            self.lexer.put_back(tok);
                            self.expect(Token::Semi)?;
                            Stmt::Expr(self.expr()?)
                        }
                    }
                } else {
                    return Err(eparse!(self, "expected expression or assignment"));
                }
            }
            tok => {
                self.lexer.put_back(tok);
                let res = Stmt::Expr(self.expr()?);
                self.expect(Token::Semi)?;
                res
            }
        })
    }

    /// Parses a declaration or initialization statement.
    fn decl_or_init_stmt(&mut self) -> Result<Stmt> {
        if let Some(tok) = self.lexer.next() {
            match tok? {
                // Get the type
                Token::Keyword(Keyword::Int) => {
                    let ident = self.ident()?;
                    if let Some(tok) = self.lexer.next() {
                       match tok? { 
                            // Distinguish between declaration and initialization
                            Token::Semi => Ok(Stmt::Decl(false, Type::Int, ident)),
                            Token::Assign => {
                                let expr = self.expr()?;
                                self.expect(Token::Semi)?;
                                Ok(Stmt::Init(false, Type::Int, ident, expr))
                            }
                            tok => Err(eparse!(self, "expected `;` or `=`, got `{}`", tok)),
                        }
                    } else {
                        Err(eparse!(self, "expected `;` or `=`"))
                    }
                }
                tok => Err(eparse!(self, "expected type, got `{}`", tok)),
            }
        } else {
            Err(eparse!(self, "expected declaration or initialization statement"))
        }
    }

    /// Parses an expression.
    fn expr(&mut self) -> Result<Expr> {
        if let Some(tok) = self.lexer.next() {
            let lhs = match tok? {
                Token::Ident(ident) => Expr::Ident(ident),
                Token::Literal(lit) => Expr::Literal(lit),
                tok => return Err(eparse!(self, "expected expression term, got `{}`", tok))
            };
            Ok(match self.lexer.next() {
                None => lhs,
                Some(Err(e)) => return Err(e),
                Some(Ok(Token::Add)) => Expr::BinOp(BinOp::Add, Box::new(lhs), Box::new(self.expr()?)),
                Some(Ok(Token::Sub)) => Expr::BinOp(BinOp::Sub, Box::new(lhs), Box::new(self.expr()?)),
                Some(Ok(tok @ Token::Semi)) => {
                    self.lexer.put_back(tok);
                    lhs
                }
                Some(Ok(tok)) => return Err(eparse!(self, "expected binary operation, got `{}`", tok)),
            })
        } else {
            Err(eparse!(self, "expected expression"))
        }
    }

    /// Parses an identifier.
    fn ident(&mut self) -> Result<Ident> {
        match self.lexer.next() {
            None => Err(eparse!(self, "expected identifier")),
            Some(Err(e)) => Err(e),
            Some(Ok(Token::Ident(ident))) => Ok(ident),
            Some(Ok(tok)) => Err(eparse!(self, "expected identifier, got `{}`", tok)),
        }
    }

    /// Expects the given token, consuming it if it is the next token in the stream and returning an error if it is not.
    fn expect(&mut self, tok: Token) -> Result<()> {
        let got = match self.lexer.next() {
            Some(got) => got?,
            None => return Err(eparse!(self, "expected `{}`", tok)),    
        };

        if got != tok {
            Err(eparse!(self, "expected `{}`, got `{}`", tok, got))
        } else {
            Ok(())
        }
    }
}

