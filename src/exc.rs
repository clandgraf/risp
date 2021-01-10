
use crate::lisp_object::{EvalError};

pub fn apply_unimpl() -> EvalError {
    EvalError::new("apply only implemented for Native, Lambda and Special Form".to_string())
}

pub fn apply_empty() -> EvalError {
    EvalError::new("apply received empty form".to_string())
}

pub fn unbound_symbol(sym: Option<&str>) -> EvalError {
    EvalError::new(format!("Unbound symbol '{}'",
                           sym.unwrap_or("~~uninterned~~")))
}

pub fn unexpected_lambda() -> EvalError {
    EvalError::new(
        format!("Unexpected lambda. This is probably an internal error."))
}

pub fn unexpected_special_form() -> EvalError {
    EvalError::new(
        format!("Unexpected special form. This is probably an internal error."))
}
