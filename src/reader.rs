use std::fmt;

use crate::lexer::{Tokens, ObjectT, StringT, Lexer};
use crate::LispObject;

const UNKNOWN_CHAR: &str = "Unexpected character.";
const UNEXPECTED_RBRACE: &str = "Right brace without matching lbrace.";
const UNEXPECTED_ENDOFSTR: &str = "Unexpected end of input while parsing string.";
const INTERNAL_ERROR: &str = "Internal Error.";

pub enum ReadError {
    UnknownCharacter((usize, usize)),
    UnexpectedRbrace((usize, usize)),
    UnexpectedEndOfString,
    InternalError,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            ReadError::UnknownCharacter(_) => UNKNOWN_CHAR,
            ReadError::UnexpectedRbrace(_) => UNEXPECTED_RBRACE,
            ReadError::UnexpectedEndOfString => UNEXPECTED_ENDOFSTR,
            ReadError::InternalError => INTERNAL_ERROR,
        })
    }
}

pub struct Reader {
    stack: Vec<Vec<LispObject>>
}

impl Reader {
    pub fn new() -> Reader {
        Reader {
            stack: vec![]
        }
    }

    pub fn partial(&mut self, prog: &mut Vec<LispObject>, input: &String) -> Result<(), ReadError> {
        let mut lexer = Lexer::new(input);
        loop {
            match self.parse_sexp(&mut lexer) {
                Ok(Some(sexp)) => prog.push(sexp),
                Ok(None) => return Ok(()),
                Err(s) => return Err(s),
            }
        }
    }

    fn parse_sexp(&mut self, lexer: &mut Lexer) -> Result<Option<LispObject>, ReadError> {
        loop {
            match lexer.next() {
                Some(Tokens::String(_))
                    => return Err(ReadError::InternalError),
                Some(Tokens::Object(ObjectT::Error))
                    => return Err(ReadError::UnknownCharacter(lexer.span())),

                None
                    => return Ok(None),

                Some(Tokens::Object(ObjectT::LBrace))
                    => self.stack.push(vec![]),
                Some(Tokens::Object(ObjectT::RBrace))
                    => match self.stack.len() {
                        0 => return Err(ReadError::UnexpectedRbrace(lexer.span())),
                        1 => return Ok(Some(self.pop_list())),
                        _ => {
                            let sxp = self.pop_list();
                            self.push_sexp(sxp);
                        }
                    },
                Some(Tokens::Object(ObjectT::Number(n)))
                    => if let Some(a) = self.handle_atom(LispObject::Number(n)) {
                        return Ok(Some(a))
                    },
                Some(Tokens::Object(ObjectT::Symbol(s)))
                    => if let Some(a) = self.handle_atom(LispObject::Symbol(s)) {
                        return Ok(Some(a))
                    },
                Some(Tokens::Object(ObjectT::StartString))
                    => return self.parse_string(lexer).map(|s| Some(s)),
            }
        }
    }

    fn handle_atom(&mut self, a: LispObject) -> Option<LispObject> {
        match self.stack.len() {
            0 => return Some(a),
            _ => self.push_sexp(a)
        }
        None
    }

    fn parse_string(&mut self, lexer: &mut Lexer) -> Result<LispObject, ReadError> {
        let mut string = String::new();
        let res = loop {
            match lexer.next() {
                Some(Tokens::Object(_))
                    => return Err(ReadError::InternalError),
                Some(Tokens::String(StringT::Error))
                    => return Err(ReadError::UnknownCharacter(lexer.span())),

                None
                    => break Err(ReadError::UnexpectedEndOfString),

                Some(Tokens::String(StringT::Text(s)))
                    => string.push_str(&s[..]),
                Some(Tokens::String(StringT::EndString))
                    => break Ok(()),
            }
        };
        res.map(|()| LispObject::String(string))
    }

    fn push_sexp(&mut self, obj: LispObject) {
        let mut l = self.stack.pop().unwrap();
        l.push(obj);
        self.stack.push(l);
    }

    fn pop_list(&mut self) -> LispObject {
        LispObject::List(self.stack.pop().unwrap())
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
