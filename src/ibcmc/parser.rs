//! The IBCMC parser.

use std::io::Result as IoResult;

use ibcmc::errors::*;
use ibcmc::ast::{Type, BinOp, Block, Stmt, Expr, Decl, StmtLine};
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

        while let Some(tok) = self.lexer.peek() {
            if tok? == Token::RBrace {
                // End of the current block
                break;
            }
            stmts.push(self.stmt()?);
        }

        Ok(Block(stmts))
    }

    /// Parses a statement.
    ///
    /// The semicolon at the end of the line will be parsed as a part of this node (or a helper function, like `stmt_after_type`), if relevant.
    fn stmt(&mut self) -> Result<StmtLine> {
        if let Some(tok) = self.lexer.next() {
            let line = self.lexer.line();
            let stmt = match tok? {
                Token::Semi => Stmt::Empty,
                tok @ Token::Keyword(Keyword::Const) | tok @ Token::Keyword(Keyword::Int) => {
                    self.lexer.put_back(tok);
                    self.stmt_after_type()?
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
                Token::LBrace => {
                    let block = self.block()?;
                    self.expect(Token::RBrace)?;
                    Stmt::Block(block)
                }
                tok => {
                    self.lexer.put_back(tok);
                    let res = Stmt::Expr(self.expr()?);
                    self.expect(Token::Semi)?;
                    res
                }
            };
            Ok(stmt.with_line(line))
        } else {
            Err(eparse!(self, "unexpected end of program"))
        }
    }

    /// Parses a variable declaration, initialization, or function definition.
    fn stmt_after_type(&mut self) -> Result<Stmt> {
        let decl = self.decl()?;
        if let Some(tok) = self.lexer.next() {
            // Distinguish between declaration, initialization, and function declaration
            Ok(match tok? { 
                Token::Semi => Stmt::Decl(decl),
                Token::Assign => {
                    let expr = self.expr()?;
                    self.expect(Token::Semi)?;
                    Stmt::Init(decl, expr)
                }
                Token::LParen => {
                    let param_list = self.param_list()?;
                    self.expect(Token::RParen)?;
                    self.expect(Token::LBrace)?;
                    let body = self.block()?;
                    self.expect(Token::RBrace)?;
                    Stmt::Function(decl, param_list, body)
                }
                tok => return Err(eparse!(self, "expected `;`, `=`, or `(`, got `{}`", tok)),
            })
        } else {
            Err(eparse!(self, "expected `;`, `=`, or `(`"))
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

    /// Parses a list of function parameters.
    fn param_list(&mut self) -> Result<Vec<Decl>> {
        let mut params = Vec::new();

        while let Some(tok) = self.lexer.peek() {
            if tok? == Token::RParen {
                break;
            }
            params.push(self.decl()?);

            if let Some(tok) = self.lexer.next() {
                match tok? {
                    Token::Comma => continue,
                    tok => {
                        self.lexer.put_back(tok);
                        break;
                    }
                }
            } else {
                return Err(eparse!(self, "unexpected end of function parameter list"));
            }
        }

        Ok(params)
    }

    /// Parses a declaration.
    fn decl(&mut self) -> Result<Decl> {
        if let Some(tok) = self.lexer.next() {
            Ok(match tok? {
                Token::Keyword(Keyword::Const) => {
                    let mut res = self.decl()?;
                    res.is_const = true;
                    res
                }
                Token::Keyword(Keyword::Int) => {
                    let name = self.ident()?;
                    Decl {
                        is_const: false,
                        ty: Type::Int,
                        name
                    }
                }
                tok => return Err(eparse!(self, "expected type, found `{}`", tok)),
            })
        } else {
            Err(eparse!(self, "expected variable declaration"))
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

