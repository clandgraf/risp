use std::fmt;

#[derive(Clone)]
pub enum SpecialForm {
    Def,
    Set,
    Fn,
    If,
    Let,
    Begin,
    Quote,
}

impl fmt::Display for SpecialForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            SpecialForm::Def => "def",
            SpecialForm::Set => "set",
            SpecialForm::Fn => "fn",
            SpecialForm::If => "if",
            SpecialForm::Let => "let",
            SpecialForm::Begin => "begin",
            SpecialForm::Quote => "quote",
        })
    }
}

pub type Symbol = u64;

pub type Sexpr = Vec<LispObject>;

pub type ParamList = (Vec<Symbol>, Option<Symbol>);

#[derive(Clone)]
pub enum LispObject {
    Bool(bool),
    SpecialForm(SpecialForm),
    Symbol(Symbol),
    String(String),
    Number(f64),
    List(Sexpr),
    Native(ParamList, Native),
    Macro(ParamList, Sexpr),
    Lambda(ParamList, Sexpr),
}

// When an error occurs during evaluation an Err(EvalError) is returned.
// - frames contains the s-expressions that eval processed, resolved from
//   function definitions.
// - trace contains the position in the current frame where the error is
//   occurred.

pub type Trace = Vec<usize>;
pub type Frame = (LispObject, Trace);

pub struct EvalError {
    pub message: String,      // Message describing the error
    pub frames: Vec<Frame>,   // Already handled frames
    pub trace: Trace,         // Current trace
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
            frames: vec![],
        }
    }

    pub fn trace(mut self, index: usize) -> EvalError {
        self.trace.push(index);
        self
    }

    pub fn frame(mut self, expr: LispObject) -> EvalError {
        self.frames.push((expr, self.trace));
        self.trace = vec![];
        self
    }
}

pub type Native = fn(&[LispObject]) -> Result<LispObject, EvalError>;

pub struct NativeDef {
    pub name: &'static str,
    pub positional: &'static [&'static str],
    pub rest: Option<&'static str>,
    pub func: Native,
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

    pub fn as_symbol(&self) -> Result<Symbol, EvalError> {
        match self {
            LispObject::Symbol(s) => Ok(*s),
            _ => Err(EvalError::new("Expected a symbol".to_string())),
        }
    }

    pub fn as_list(&self) -> Result<Sexpr, EvalError> {
        match self {
            LispObject::List(l) => Ok(l.clone()),
            _ => Err(EvalError::new("Expected a list".to_string())),
        }
    }

    pub fn into_list(self) -> Result<Sexpr, EvalError> {
        match self {
            LispObject::List(l) => Ok(l),
            _ => Err(EvalError::new("Expected a list".to_string())),
        }
    }
}
