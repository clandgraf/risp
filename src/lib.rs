use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;

mod lexer;
pub mod reader;
mod native;

type Native = fn(&[LispObject]) -> Result<LispObject, EvalError>;

#[derive(Clone)]
pub enum SpecialForm {
    If,
    Def,
}

impl fmt::Display for SpecialForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            SpecialForm::If => "if",
            SpecialForm::Def => "def",
        })
    }
}

type Symbol = u64;

pub struct Symbols {
    registry: HashMap<String, Symbol>,
    reverse: HashMap<Symbol, String>,
    next_id: Symbol,
}

impl Symbols {
    pub fn new() -> Symbols {
        Symbols {
            registry: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn intern(&mut self, name: &str) -> Symbol {
        match self.registry.entry(name.to_string()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(_) => {
                self.next_id += 1;
                self.registry.insert(name.to_string(), self.next_id);
                self.reverse.insert(self.next_id, name.to_string());
                self.next_id
            }
        }
    }

    pub fn as_string(&self, sym: &Symbol) -> Option<&str> {
        self.reverse.get(sym).map(|s| &s[..])
    }

    fn form_to_string(&self, l: &Vec<LispObject>) -> String {
        l.iter()
            .map(|o| self.serialize_object(o))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn serialize_object(&self, obj: &LispObject) -> String {
        match obj {
            LispObject::Symbol(s) => format!("{}", self.as_string(s)
                                             .unwrap_or("~~uninterned~~")),
            LispObject::List(l) => format!("({})", self.form_to_string(l)),
            _ => obj.to_string(),
        }
    }
}

pub struct EvalError {
    pub message: String,
    pub trace: Vec<usize>,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl EvalError {
    pub fn new(message: String) -> EvalError {
        EvalError {
            message: message,
            trace: vec![],
        }
    }

    pub fn trace(mut self, index: usize) -> EvalError {
        self.trace.push(index);
        self
    }
}

#[derive(Clone)]
pub enum LispObject {
    SpecialForm(SpecialForm),
    Symbol(Symbol),
    String(String),
    Number(f64),
    List(Vec<LispObject>),
    Native(Native),
}

fn form_to_string(l: &Vec<LispObject>) -> String {
    l.iter()
        .map(|o| o.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

// impl fmt::Display for LispObject {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", match self {
//             LispObject::Symbol(s) => format!("{}:{}", s.to_string(), "sym"),
//             LispObject::String(s) => format!("\"{}\":{}", s, "str"),
//             LispObject::Number(n) => format!("{}:{}", n.to_string(), "num"),
//             LispObject::List(l) => format!("({})", form_to_string(l)),
//             LispObject::Native(_) => "~~native~~".to_string(),
//         })
//     }
// }

impl fmt::Display for LispObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            LispObject::SpecialForm(sf) => format!("{}", sf),
            LispObject::Symbol(s) => format!("{}", s),
            LispObject::String(s) => format!("\"{}\"", s),
            LispObject::Number(n) => format!("{}", n.to_string()),
            LispObject::List(l) => format!("({})", form_to_string(l)),
            LispObject::Native(_) => "~~native~~".to_string(),
        })
    }
}

pub fn as_symbol(obj: &LispObject) -> Result<String, EvalError> {
    match obj {
        LispObject::Symbol(s) => Ok(s.to_string()),
        _ => Err(EvalError::new("Expected a symbol".to_string())),
    }
}

pub struct ArgError {
    pub message: String,
    pub index: usize,
}

pub fn as_numbers(objects: &[LispObject]) -> Result<Vec<f64>, ArgError> {
    objects
        .iter().enumerate()
        .map(|(index, object)| {
            match expect_number(object) {
                Err(message) => Err(ArgError{message, index}),
                Ok(n) => Ok(n),
            }
        })
        .collect()
}

pub fn expect_number(obj: &LispObject) -> Result<f64, String> {
    match obj {
        LispObject::Number(n) => Ok(*n),
        _ => Err("Expected a number".to_string()),
    }
}

pub struct Env<'a> {
    vars: HashMap<Symbol, LispObject>,
    parent: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    fn root() -> Env<'static> {
        Env {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn derive(&self) -> Env {
        Env {
            vars: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn set(&mut self, key: Symbol, value: LispObject) {
        self.vars.insert(key, value);
    }

    pub fn resolve(&self, key: &Symbol) -> Option<&LispObject> {
        match self.vars.get(key) {
            Some(value) => Some(value),
            None => match self.parent {
                Some(scope) => scope.resolve(key),
                None => None,
            }
        }
    }
}

fn set_native(env: &mut Env, key: Symbol, value: Native) {
    env.set(key, LispObject::Native(value));
}

fn set_special(sym: &mut Symbols, env: &mut Env, sf: SpecialForm) {
    env.set(sym.intern(&sf.to_string()),
            LispObject::SpecialForm(sf));
}

pub fn create_root(symbols: &mut Symbols) -> Env<'static> {
    let mut root = Env::root();
    set_special(symbols, &mut root, SpecialForm::If);
    set_special(symbols, &mut root, SpecialForm::Def);
    set_native(&mut root, symbols.intern("+"), native::add);
    set_native(&mut root, symbols.intern("*"), native::multiply);
    root
}
