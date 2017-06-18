//! The IBCMC lexer.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Result as IoResult;

use itertools::{self, PutBack};

use ibcmc::errors::*;

/// A token.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Token {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,

    /// An identifier.
    Ident(Ident),
    /// A keyword.
    Keyword(Keyword),
    /// A literal.
    Literal(Literal),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &Token::Add => write!(f, "+"),
            &Token::Sub => write!(f, "-"),
            &Token::Assign => write!(f, "="),
            &Token::AddAssign => write!(f, "+="),
            &Token::SubAssign => write!(f, "-="),
            &Token::Semi => write!(f, ";"),
            &Token::Comma => write!(f, ","),
            &Token::LParen => write!(f, "("),
            &Token::RParen => write!(f, ")"),
            &Token::LBrace => write!(f, "{{"),
            &Token::RBrace => write!(f, "}}"),
            &Token::Ident(ref ident) => write!(f, "{}", ident),
            &Token::Keyword(ref keyword) => write!(f, "{}", keyword),
            &Token::Literal(ref literal) => write!(f, "{}", literal),
        }
    }
}

/// An identifier.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Ident(pub String);

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "ident({})", self.0)
    }
}

/// A keyword.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Keyword {
    /// `const`
    Const,
    /// `int`
    Int,
    /// `void`
    Void,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Keyword::Const => write!(f, "const"),
            Keyword::Int => write!(f, "int"),
            Keyword::Void => write!(f, "void"),
        }
    }
}

/// A literal.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Literal {
    /// An integer literal.
    Int(u16),
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Literal::Int(n) => write!(f, "int_lit({})", n),
        }
    }
}

/// Represents the state of the lexer.
pub struct Lexer<I>
    where I: Iterator<Item = IoResult<u8>>
{
    input: PutBack<I>,
    line: usize,
    /// Buffer of tokens (for use with the `put_back` method).
    buf: Vec<Token>,
}

impl<I> Lexer<I>
    where I: Iterator<Item = IoResult<u8>>
{
    /// Creates a new lexer from the given input iterator.
    pub fn new<J>(input: J) -> Lexer<I>
        where J: IntoIterator<Item = IoResult<u8>, IntoIter = I>
    {
        Lexer {
            input: itertools::put_back(input),
            line: 1,
            buf: Vec::new(),
        }
    }

    /// Returns the current line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Puts the given token back into the lexer for later access.
    pub fn put_back(&mut self, tok: Token) {
        self.buf.push(tok);
    }

    /// Returns the next token in the stream without consuming it.
    pub fn peek(&mut self) -> Option<Result<Token>> {
        match self.next() {
            Some(Ok(tok)) => {
                self.put_back(tok.clone());
                Some(Ok(tok))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    /// Helper method for parsing an integer literal.
    fn parse_int_lit(&mut self) -> Result<Token> {
        let mut int = 0;

        while let Some(res) = self.input.next() {
            match res.chain_err(|| {
                                    ErrorKind::Lexer("could not read lexer input".into(), self.line)
                                })? {
                digit @ b'0'...b'9' => int = 10 * int + (digit - b'0') as u16,
                other => {
                    self.input.put_back(Ok(other));
                    break;
                }
            }
        }

        Ok(Token::Literal(Literal::Int(int)))
    }

    /// Helper method for parsing a word (identfier or keyword).
    fn parse_word(&mut self) -> Result<Token> {
        let mut word = String::new();

        while let Some(res) = self.input.next() {
            match res.chain_err(|| {
                                    ErrorKind::Lexer("could not read lexer input".into(), self.line)
                                })? {
                letter @ b'A'...b'Z' |
                letter @ b'a'...b'z' |
                letter @ b'0'...b'9' => word.push(letter as char),
                other => {
                    self.input.put_back(Ok(other));
                    break;
                }
            }
        }

        // Check to see if we have a keyword
        Ok(match word.as_str() {
               "const" => Token::Keyword(Keyword::Const),
               "int" => Token::Keyword(Keyword::Int),
               "void" => Token::Keyword(Keyword::Void),
               _ => Token::Ident(Ident(word)),
           })
    }
}

impl<I> Iterator for Lexer<I>
    where I: Iterator<Item = IoResult<u8>>
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
        if let Some(tok) = self.buf.pop() {
            return Some(Ok(tok));
        }

        match self.input.next() {
            Some(Ok(b)) => {
                Some(Ok(match b {
                            b'+' => {
                                match self.input.next() {
                                    Some(Ok(b'=')) => Token::AddAssign,
                                    Some(res) => {
                                        self.input.put_back(res);
                                        Token::Add
                                    }
                                    None => Token::Add,
                                }
                            }
                            b'-' => {
                                match self.input.next() {
                                    Some(Ok(b'=')) => Token::SubAssign,
                                    Some(res) => {
                                        self.input.put_back(res);
                                        Token::Sub
                                    }
                                    None => Token::Sub,
                                }
                            }
                            b'=' => Token::Assign,
                            b';' => Token::Semi,
                            b',' => Token::Comma,
                            b'(' => Token::LParen,
                            b')' => Token::RParen,
                            b'{' => Token::LBrace,
                            b'}' => Token::RBrace,
                            b'0'...b'9' => {
                                self.input.put_back(Ok(b));
                                return Some(self.parse_int_lit());
                            }
                            b'\n' => {
                                self.line += 1;
                                return self.next();
                            }
                            b' ' | b'\t' | b'\r' => return self.next(),
                            b'A'...b'Z' | b'a'...b'z' => {
                                self.input.put_back(Ok(b));
                                return Some(self.parse_word());
                            }
                            _ => {
                                return Some(Err(ErrorKind::Lexer(format!("unknown token `{}`",
                                                                         b as char),
                                                                 self.line)
                                                    .into()))
                            }
                        }))
            }
            Some(Err(e)) => {
                Some(Err(Error::with_chain(e,
                                           ErrorKind::Lexer("could not read lexer input".into(),
                                                            self.line))))
            }
            None => None,
        }
    }
}

