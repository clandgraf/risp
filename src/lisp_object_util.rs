use crate::lisp_object::{
    EvalError,
    LispObject,
    Symbol,
};

pub fn assert_exact_form_args(form: &[LispObject], len: usize, description: fn() -> String)
                   -> Result<(), EvalError> {
    let actual_len = form.len();
    if actual_len != len {
        Err(EvalError::new(format!("{} requires exactly {} arguments, got {}",
                                   description(), len, actual_len)))
    } else {
        Ok(())
    }
}

pub fn assert_min_form_args(form: &[LispObject], len: usize, description: fn() -> String)
                   -> Result<(), EvalError> {
    let actual_len = form.len();
    if actual_len < len {
        Err(EvalError::new(format!("{} requires at least {} arguments, got {}",
                                   description(), len, actual_len)))
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
