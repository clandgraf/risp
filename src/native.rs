use crate::{
    lisp_object::{
        EvalError,
        LispObject,
        NativeDef,
    },
    lisp_object_util::{
        as_numbers,
    },
};

fn add(args: &[LispObject]) -> Result<LispObject, EvalError> {
    let terms = args[0].as_list()?;
    as_numbers(&terms)
        .map(|args| LispObject::Number(args.iter().fold(0.0, |sum, a| sum + a)))
        .map_err(|(err, index)| err.trace(index + 1))
}

pub const ADD: NativeDef = NativeDef {
    name: "+",
    positional: &[],
    rest: Some("terms"),
    func: add,
};

fn multiply(args: &[LispObject]) -> Result<LispObject, EvalError> {
    let factors = args[0].as_list()?;
    as_numbers(&factors)
        .map(|args| LispObject::Number(args.iter().fold(1.0, |sum, a| sum * a)))
        .map_err(|(err, index)| err.trace(index + 1))
}

pub const MULTIPLY: NativeDef = NativeDef {
    name: "*",
    positional: &[],
    rest: Some("factors"),
    func: multiply
};

fn subtract(args: &[LispObject]) -> Result<LispObject, EvalError> {
    let min = args[0].as_number()
        .map_err(|err| err.trace(1))?;
    let subs = args[1].as_list()?;
    let sub = as_numbers(&subs)
        .map(|args| args.iter().fold(0.0, |sum, a| sum + a))
        .map_err(|(err, index)| err.trace(index + 2))?;
    Ok(LispObject::Number(min - sub))
}

pub const SUBTRACT: NativeDef = NativeDef {
    name: "-",
    positional: &["min"],
    rest: Some("subs"),
    func: subtract,
};

fn equal(args: &[LispObject]) -> Result<LispObject, EvalError> {
    match args[0] {
        LispObject::Number(op0) => {
            let op1 = args[1].as_number()
                .map_err(|e| e.trace(2))?;
            Ok(LispObject::Bool(op0 == op1))
        }
        _ => Err(EvalError::new("equal not implemented for type".to_string()).trace(1)),
    }
}

pub const EQUAL: NativeDef = NativeDef {
    name: "=",
    positional: &["o1", "o2"],
    rest: None,
    func: equal,
};

fn first(args: &[LispObject]) -> Result<LispObject, EvalError> {
    let lst = args[0].as_list()?;
    Ok(lst[0].clone())
}

pub const FIRST: NativeDef = NativeDef {
    name: "first",
    positional: &["lst"],
    rest: None,
    func: first,
};

fn rest(args: &[LispObject]) -> Result<LispObject, EvalError> {
    let lst = args[0].as_list()?;
    let res = if lst.len() > 0 {
        lst[1..].to_vec()
    } else {
        vec![]
    };
    Ok(LispObject::List(res))
}

pub const REST: NativeDef = NativeDef {
    name: "rest",
    positional: &["lst"],
    rest: None,
    func: rest,
};
