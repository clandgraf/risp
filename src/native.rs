use crate::{
    lisp_object::{
        EvalError,
        LispObject,
    },
    lisp_object_util::{
        as_numbers,
        assert_min_form_args,
        assert_exact_form_args,
    },
};

pub fn add(args: &[LispObject]) -> Result<LispObject, EvalError> {
    as_numbers(args)
        .map(|args| LispObject::Number(args.iter().fold(0.0, |sum, a| sum + a)))
        .map_err(|(err, index)| err.trace(index + 1))
}

pub fn minus(args: &[LispObject]) -> Result<LispObject, EvalError> {
    assert_min_form_args(args, 2, || "inbuild '-'".to_string())?;
    let min = args[0].as_number()
        .map_err(|err| err.trace(1))?;
    let sub = as_numbers(&args[1..])
        .map(|args| args.iter().fold(0.0, |sum, a| sum + a))
        .map_err(|(err, index)| err.trace(index + 2))?;
    Ok(LispObject::Number(min - sub))
}

pub fn multiply(args: &[LispObject]) -> Result<LispObject, EvalError> {
    as_numbers(args)
        .map(|args| LispObject::Number(args.iter().fold(1.0, |sum, a| sum * a)))
        .map_err(|(err, index)| err.trace(index + 1))
}

pub fn equal(args: &[LispObject]) -> Result<LispObject, EvalError> {
    assert_exact_form_args(args, 2, || "inbuild '-'".to_string())?;
    match args[0] {
        LispObject::Number(op0) => {
            let op1 = args[1].as_number()
                .map_err(|e| e.trace(2))?;
            Ok(LispObject::Bool(op0 == op1))
        }
        _ => Err(EvalError::new("equal not implemented for type".to_string()).trace(1)),
    }
}
