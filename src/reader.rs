use std::fmt;

use crate::{
    lexer::{Tokens, ObjectT, StringT, Lexer},
    lisp_object::LispObject,
    env::Symbols,
};

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

pub enum ReaderFrame {
    Sexpr(Vec<LispObject>),
    Quote,
    QuasiQuote,
    Unquote,
    UnquoteSplice,
}

pub struct Reader {
    stack: Vec<ReaderFrame>
}

impl Reader {
    pub fn new() -> Reader {
        Reader {
            stack: vec![]
        }
    }

    pub fn partial(&mut self, symbols: &mut Symbols, prog: &mut Vec<LispObject>, input: &str)
                   -> Result<(), ReadError> {
        let mut lexer = Lexer::new(input);
        loop {
            match self.parse_sexp(symbols, &mut lexer) {
                Ok(Some(sexp)) => prog.push(sexp),
                Ok(None) => return Ok(()),
                Err(s) => return Err(s),
            }
        }
    }

    fn parse_sexp(&mut self, symbols: &mut Symbols, lexer: &mut Lexer)
                  -> Result<Option<LispObject>, ReadError> {
        loop {
            match lexer.next() {
                Some(Tokens::String(_))
                    => return Err(ReadError::InternalError),
                Some(Tokens::Object(ObjectT::Error))
                    => return Err(ReadError::UnknownCharacter(lexer.span())),

                None
                    => return Ok(None),

                // Starting an expression that is not an atom. This will be built on the
                // stack and completed either by encountering the associated expression of
                // the quote or the closing brace.
                Some(Tokens::Object(ObjectT::Quote))
                    => self.stack.push(ReaderFrame::Quote),
                Some(Tokens::Object(ObjectT::QuasiQuote))
                    => self.stack.push(ReaderFrame::QuasiQuote),
                Some(Tokens::Object(ObjectT::Unquote))
                    => self.stack.push(ReaderFrame::Unquote),
                Some(Tokens::Object(ObjectT::UnquoteSplice))
                    => self.stack.push(ReaderFrame::UnquoteSplice),
                Some(Tokens::Object(ObjectT::LBrace))
                    => self.stack.push(ReaderFrame::Sexpr(vec![])),

                // Finishing an expression
                Some(Tokens::Object(ObjectT::RBrace))
                    => {
                        let obj = self.pop_list(lexer)?;
                        if let Some(a) = self.handle_obj(symbols, obj) {
                            return Ok(Some(a))
                        }
                    },
                Some(Tokens::Object(ObjectT::Symbol(s)))
                    => {
                        let obj = symbols.symbol(&s);
                        if let Some(a) = self.handle_obj(symbols, obj) {
                            return Ok(Some(a))
                        }
                    },
                Some(Tokens::Object(ObjectT::StartString))
                    => {
                        let obj = self.parse_string(lexer)?;
                        if let Some(a) = self.handle_obj(symbols, obj) {
                            return Ok(Some(a))
                        }
                    }
                Some(Tokens::Object(ObjectT::True))
                    => if let Some(a) = self.handle_obj(symbols, LispObject::Bool(true)) {
                        return Ok(Some(a))
                    },
                Some(Tokens::Object(ObjectT::False))
                    => if let Some(a) = self.handle_obj(symbols, LispObject::Bool(false)) {
                        return Ok(Some(a))
                    },
                Some(Tokens::Object(ObjectT::Number(n)))
                    => if let Some(a) = self.handle_obj(symbols, LispObject::Number(n)) {
                        return Ok(Some(a))
                    },
            }
        }
    }

    fn handle_obj(&mut self, symbols: &mut Symbols, obj: LispObject) -> Option<LispObject> {
        let mut obj = obj;
        loop {
            match self.stack.pop() {
                Some(frame) => match frame {
                    ReaderFrame::Quote          => obj = symbols.quote(obj),
                    ReaderFrame::QuasiQuote     => obj = symbols.quasi_quote(obj),
                    ReaderFrame::Unquote        => obj = symbols.unquote(obj),
                    ReaderFrame::UnquoteSplice  => obj = symbols.unquote_splice(obj),
                    ReaderFrame::Sexpr(mut lst) => {
                        lst.push(obj);
                        self.stack.push(ReaderFrame::Sexpr(lst));
                        return None
                    },
                },
                None => return Some(obj)
            }
        }
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

    fn pop_list(&mut self, lexer: &mut Lexer) -> Result<LispObject, ReadError> {
        if let Some(ReaderFrame::Sexpr(lst)) = self.stack.pop() {
            Ok(LispObject::List(lst))
        } else {
            Err(ReadError::UnexpectedRbrace(lexer.span()))
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}
