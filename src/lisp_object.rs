use std::fmt;

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

pub type Native = fn(&[LispObject]) -> Result<LispObject, EvalError>;

#[derive(Clone)]
pub enum SpecialForm {
    Def,
    Fn,
    If,
    Let,
}

impl fmt::Display for SpecialForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            SpecialForm::Def => "def",
            SpecialForm::Fn => "fn",
            SpecialForm::If => "if",
            SpecialForm::Let => "let",
        })
    }
}

pub type Symbol = u64;

#[derive(Clone)]
pub enum LispObject {
    Bool(bool),
    SpecialForm(SpecialForm),
    Symbol(Symbol),
    String(String),
    Number(f64),
    List(Vec<LispObject>),
    Native(Native),
}

impl LispObject {
    pub fn as_bool(&self) -> Result<bool, EvalError> {
        match self {
            LispObject::Bool(b) => Ok(*b),
            _ => Err(EvalError::new("Expected a bool".to_string())),
        }
    }

    pub fn as_number(&self) -> Result<f64, EvalError> {
        match self {
            LispObject::Number(n) => Ok(*n),
            _ => Err(EvalError::new("Expected a number".to_string())),
        }
    }

    pub fn as_symbol(obj: &LispObject) -> Result<Symbol, EvalError> {
        match obj {
            LispObject::Symbol(s) => Ok(*s),
            _ => Err(EvalError::new("Expected a symbol".to_string())),
        }
    }
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

fn form_to_string(l: &Vec<LispObject>) -> String {
    l.iter()
        .map(|o| o.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

impl fmt::Display for LispObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            LispObject::Bool(true) => "#t".to_string(),
            LispObject::Bool(false) => "#f".to_string(),
            LispObject::SpecialForm(sf) => format!("{}", sf),
            LispObject::Symbol(s) => format!("{}", s),
            LispObject::String(s) => format!("\"{}\"", s),
            LispObject::Number(n) => format!("{}", n.to_string()),
            LispObject::List(l) => format!("({})", form_to_string(l)),
            LispObject::Native(_) => "~~native~~".to_string(),
        })
    }
}
