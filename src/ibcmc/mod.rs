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
        }
    }
}

pub mod lexer;

