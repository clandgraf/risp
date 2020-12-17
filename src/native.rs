use crate::{LispObject, EvalError, as_numbers};

pub fn add(args: &[LispObject]) -> Result<LispObject, EvalError> {
    as_numbers(args)
        .map(|args| LispObject::Number(args.iter().fold(0.0, |sum, a| sum + a)))
        .map_err(|err| EvalError::new(err.message).trace(err.index + 1))
}

pub fn multiply(args: &[LispObject]) -> Result<LispObject, EvalError> {
    as_numbers(args)
        .map(|args| LispObject::Number(args.iter().fold(1.0, |sum, a| sum * a)))
        .map_err(|err| EvalError::new(err.message).trace(err.index + 1))
}
