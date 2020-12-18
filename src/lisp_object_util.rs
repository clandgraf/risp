use crate::lisp_object::{
    EvalError,
    LispObject,
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
                Err(message) => Err((message, index)),
                Ok(n) => Ok(n),
            }
        })
        .collect()
}
