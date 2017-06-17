//! The IBCMC lexer.

use std::io::Result as IoResult;

use itertools::{self, PutBack};

use ibcmc::errors::*;

/// A token.
#[derive(Clone,Debug)]
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

    /// An identifier.
    Ident(String),
    /// A keyword.
    Keyword(Keyword),
    /// A literal.
    Literal(Literal),
}

/// A keyword.
#[derive(Clone,Debug)]
pub enum Keyword {
    /// `const`
    Const,
    /// `int`
    Int,
}

/// A literal.
#[derive(Clone,Debug)]
pub enum Literal {
    /// An integer literal.
    Int(u16),
}

/// Represents the state of the lexer.
pub struct Lexer<I>
    where I: Iterator<Item = IoResult<u8>>
{
    input: PutBack<I>,
    line: usize,
}

impl<I> Lexer<I>
    where I: Iterator<Item = IoResult<u8>>
{
    /// Creates a new lexer from the given input.
    pub fn new<J>(input: J) -> Lexer<I>
        where J: IntoIterator<Item = IoResult<u8>, IntoIter = I>
    {
        Lexer {
            input: itertools::put_back(input),
            line: 1,
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
                letter @ b'a'...b'z' => word.push(letter as char),
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
               _ => Token::Ident(word),
           })
    }
}

impl<I> Iterator for Lexer<I>
    where I: Iterator<Item = IoResult<u8>>
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
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

