use logos::{Logos, Lexer as LLexer};

#[derive(Logos, Clone, Debug, PartialEq)]
pub enum ObjectT {
    #[token("#t")]
    True,
    #[token("#f")]
    False,
    #[token("(", priority = 4)]
    LBrace,
    #[token(")", priority = 4)]
    RBrace,
    #[regex("-?(([0-9]*\\.[0-9]+|[0-9]+))", |lex| lex.slice().parse(), priority = 3)]
    Number(f64),
    #[token("\"", priority = 2)]
    StartString,
    #[regex("[^\\s\\(\\)]+", |lex| lex.slice().to_string(), priority = 1)]
    Symbol(String),
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Logos, Clone, Debug, PartialEq)]
pub enum StringT {
    #[error]
    Error,
    #[regex(r#"[^\\"]+"#, |lex| lex.slice().to_string())]
    Text(String),
    #[token("\"")]
    EndString,
}

enum Modes<'a> {
    Object(LLexer<'a, ObjectT>),
    String(LLexer<'a, StringT>),
}

pub enum Tokens {
    Object(ObjectT),
    String(StringT),
}

pub struct Lexer<'a> {
    mode: Modes<'a>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Tokens;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.mode {
            Modes::Object(lex) => {
                let t = lex.next();
                if t == Some(ObjectT::StartString) {
                    self.mode = Modes::String(lex.to_owned().morph());
                }
                t.map(Tokens::Object)
            },
            Modes::String(lex) => {
                let t = lex.next();
                if t == Some(StringT::EndString) {
                    self.mode = Modes::Object(lex.to_owned().morph());
                }
                t.map(Tokens::String)
            }
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &String) -> Lexer {
        Lexer {
            mode: Modes::Object(ObjectT::lexer(&input[..]))
        }
    }

    pub fn span(&self) -> (usize, usize) {
        match &self.mode {
            Modes::Object(lex) => (lex.span().start, lex.span().end),
            Modes::String(lex) => (lex.span().start, lex.span().end),
        }
    }
}
