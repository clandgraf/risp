use logos::{Logos, Lexer};
use std::fmt;

#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[token("(")]
    LBrace,
    #[token(")")]
    RBrace,
    #[regex("[a-zA-Z]+", |lex| lex.slice().to_string())]
    Symbol(String),
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Clone)]
pub enum SExp {
    Symbol(String),
    List(Vec<SExp>),
}

const UNKNOWN_CHAR: &str = "Unexpected character.";
const UNEXPECTED_RBRACE: &str = "Right brace without matching lbrace.";

pub enum ParseError {
    UnknownCharacter,
    UnexpectedRbrace,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            ParseError::UnknownCharacter => UNKNOWN_CHAR,
            ParseError::UnexpectedRbrace => UNEXPECTED_RBRACE,
        })
    }
}

pub struct Parser {
    stack: Vec<Vec<SExp>>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            stack: vec![]
        }
    }

    pub fn partial(&mut self, prog: &mut Vec<SExp>, input: &String) -> Result<(), ParseError> {
        let mut lex = Token::lexer(&input[..]);
        loop {
            match self.parse_sexp(&mut lex) {
                Ok(Some(sexp)) => prog.push(sexp),
                Ok(None) => return Ok(()),
                Err(s) => return Err(s),
            }
        }
    }

    fn parse_sexp(&mut self, lexer: &mut Lexer<Token>) -> Result<Option<SExp>, ParseError> {
        loop {
            match lexer.next() {
                Some(Token::Error)
                    => return Err(ParseError::UnknownCharacter),
                None
                    => return Ok(None),
                Some(Token::LBrace)
                    => self.stack.push(vec![]),
                Some(Token::RBrace)
                    => match self.stack.len() {
                        0 => return Err(ParseError::UnexpectedRbrace),
                        1 => return Ok(Some(self.pop_list())),
                        _ => {
                            let sxp = self.pop_list();
                            self.push_sexp(sxp);
                        }
                    },
                Some(Token::Symbol(s))
                    => match self.stack.len() {
                        0 => return Ok(Some(SExp::Symbol(s))),
                        _ => self.push_sexp(SExp::Symbol(s))
                    },
            }
        }
    }

    fn push_sexp(&mut self, sexp: SExp) {
        let mut l = self.stack.pop().unwrap();
        l.push(sexp);
        self.stack.push(l);
    }

    fn pop_list(&mut self) -> SExp {
        SExp::List(self.stack.pop().unwrap())
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
