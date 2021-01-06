use crate::lisp_object::{
    EvalError,
    LispObject,
    Symbol,
};

pub enum Match {
    Exact,
    Min,
}

pub fn assert_args(m: Match, form: &[LispObject], len: usize, description: fn() -> String)
                   -> Result<(), EvalError> {
    let actual_len = form.len();
    let pred = match m {
        Match::Exact => actual_len != len,
        Match::Min => actual_len < len,
    };
    if pred {
        let s = match m {
            Match::Exact => "exactly",
            Match::Min   => "at least",
        };
        Err(EvalError::new(format!("{} requires {} {} arguments, got {}",
                                   description(), s, len, actual_len)))
    } else {
        Ok(())
    }
}

pub fn as_numbers(objects: &[LispObject]) -> Result<Vec<f64>, (EvalError, usize)> {
    objects
        .iter().enumerate()
        .map(|(index, object)| {
            match object.as_number() {
                Err(e) => Err((e, index)),
                Ok(n) => Ok(n),
            }
        })
        .collect()
}

pub fn as_symbols(objects: &[LispObject]) -> Result<Vec<Symbol>, (EvalError, usize)> {
    objects
        .iter().enumerate()
        .map(|(index, object)| {
            match object.as_symbol() {
                Err(e) => Err((e, index)),
                Ok(n) => Ok(n),
            }
        })
        .collect()
}
