//! A simple language for the IBCM "architecture" which resembles a stripped-down version of C.

pub mod errors {
    //! The errors for the IBCMC compiler.

    error_chain!{
        errors {
            /// A lexer error.
            Lexer(s: String, n: usize) {
                description("lexer error")
                display("lexer error on line {}: {}", n, s)
            }

            /// A parser error.
            Parser(s: String, n: usize) {
                description("parser error")
                display("parser error on line {}: {}", n, s)
            }
        }
    }
}

pub mod ast;
pub mod lexer;
pub mod parser;

pub use self::lexer::Lexer;
pub use self::parser::Parser;

#[cfg(test)]
mod tests {
    use super::*;
    use super::ast::*;
    use super::errors::*;
    use super::lexer::*;
    use std::io::{Read, Cursor};

    fn lex(input: &[u8]) -> Vec<Token> {
        Lexer::new(Cursor::new(input).bytes())
            .collect::<Result<Vec<_>>>()
            .unwrap()
    }

    fn parse(input: &[u8]) -> Block {
        Parser::parse_from_lexer(Lexer::new(Cursor::new(input).bytes())).unwrap()
    }

    #[test]
    fn tokens() {
        // Check to make sure the lexer can parse all tokens correctly
        let tokens = b"const int i = 3;
        void function(int p1, int p2) {
            local += 5 + 2;
            k -= 6 - 1;
        }";
        let lexed = lex(tokens);

        assert_eq!(lexed,
                   [Token::Keyword(Keyword::Const),
                    Token::Keyword(Keyword::Int),
                    Token::Ident(Ident("i".into())),
                    Token::Assign,
                    Token::Literal(Literal::Int(3)),
                    Token::Semi,
                    Token::Keyword(Keyword::Void),
                    Token::Ident(Ident("function".into())),
                    Token::LParen,
                    Token::Keyword(Keyword::Int),
                    Token::Ident(Ident("p1".into())),
                    Token::Comma,
                    Token::Keyword(Keyword::Int),
                    Token::Ident(Ident("p2".into())),
                    Token::RParen,
                    Token::LBrace,
                    Token::Ident(Ident("local".into())),
                    Token::AddAssign,
                    Token::Literal(Literal::Int(5)),
                    Token::Add,
                    Token::Literal(Literal::Int(2)),
                    Token::Semi,
                    Token::Ident(Ident("k".into())),
                    Token::SubAssign,
                    Token::Literal(Literal::Int(6)),
                    Token::Sub,
                    Token::Literal(Literal::Int(1)),
                    Token::Semi,
                    Token::RBrace]);
    }

    #[test]
    fn assignment() {
        let prog = b"i = 2;
        j += 3;
        k -= 4;";
        let parsed = parse(prog);

        assert_eq!(parsed,
                   Block(vec![Stmt::Assign(Ident("i".into()), Expr::Literal(Literal::Int(2))),
                              Stmt::CompoundAssign(Ident("j".into()),
                                                   BinOp::Add,
                                                   Expr::Literal(Literal::Int(3))),
                              Stmt::CompoundAssign(Ident("k".into()),
                                                   BinOp::Sub,
                                                   Expr::Literal(Literal::Int(4)))]));
    }
}

