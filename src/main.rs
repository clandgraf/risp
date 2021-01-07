use ansi_term::Colour::{Blue, Red};
use rustyline::{error::ReadlineError, Editor};
use rustyline;
use std::fmt;
use std::iter;

use lisp::{
    lisp_object::{
        Symbol,
        ParamList,
        EvalError,
        LispObject,
        SpecialForm,
    },
    lisp_object_util::{
        Match,
        assert_args,
        as_symbols,
    },
    reader::{
        Reader,
        ReadError,
    },
    Env,
    Symbols,
    create_root
};

fn print_underline(start: usize, end: usize) {
    eprintln!(" {} {}{}", Blue.paint("|"), " ".repeat(start), Red.paint("^".repeat(end - start)));
}

fn print_range(input: &String, start: usize, end: usize) {
    eprintln!(" {} {}", Blue.paint("|"), input);
    print_underline(start, end);
}

fn print_message(displayable: &dyn fmt::Display) {
    eprintln!("{}: {}", Red.paint("Error"), displayable);
}

fn handle_read_error(input: &String, result: Result<(), ReadError>) -> Result<(), ReadError> {
    if let Err(e) = result {
        match e {
            ReadError::UnknownCharacter((start, end)) => {
                print_message(&e);
                print_range(input, start, end);
            },
            ReadError::UnexpectedRbrace((start, end)) => {
                print_message(&e);
                print_range(input, start, end);
            },
            ReadError::UnexpectedEndOfString =>
                print_message(&e),
            ReadError::InternalError =>
                return Err(ReadError::InternalError),
        }
    }
    Ok(())
}

fn handle_failed_form(sym: &Symbols, form: &LispObject, stack: &[usize]) -> (String, usize, usize) {
    if stack.len() == 0 {
        let string = sym.serialize_object(form);
        let len = string.len();
        (string, 0, len)
    } else {
        match form {
            LispObject::List(l) => {
                let stack_len = stack.len() - 1;
                let offset = stack[stack_len];
                let mut start = 0;
                let mut end = 0;
                let mut string = "(".to_string();
                for (index, object) in l.iter().enumerate() {
                    if index == offset {
                        let (s, off0, off1) = handle_failed_form(sym, object, &stack[0..stack_len]);
                        start = off0 + string.len();
                        end = off1 + string.len();
                        string.push_str(&s);
                    } else {
                        string.push_str(&sym.serialize_object(object));
                    }
                    if index == l.len() - 1 {
                        string.push_str(")");
                    } else {
                        string.push_str(" ");
                    }
                }
                (string, start, end)
            },
            _ => handle_failed_form(sym, form, &stack[0..0])
        }
    }
}

fn handle_eval_error(sym: &Symbols, error: EvalError) {
    print_message(&error);
    for (form, trace) in error.frames {
        let (string, start, end) = handle_failed_form(sym, &form, &trace);
        print_range(&string, start, end);
    }
}

fn in_scope<T, F>(
    env: &mut Env,
    binding: Option<Vec<(Symbol,LispObject)>>,
    mut f: F
) -> Result<T, EvalError>
where F: FnMut(&mut Env) -> Result<T, EvalError> {
    env.push_scope();
    binding.map_or((),
        |b| b.into_iter().for_each(|(sym, value)| env.set(sym, value)));
    let res = f(env);
    env.pop_scope();
    res
}

fn bind_param_list<'a>(sym: &Symbols, env: &mut Env, params: &ParamList, tail: &[LispObject])
                   -> Result<Vec<(Symbol, LispObject)>, EvalError> {
    // Check Validity of Arguments
    let m = match params.1 {
        None    => Match::Exact,
        Some(_) => Match::Min,
    };
    assert_args(m, tail, params.0.len(),
                || format!("param list {}", sym.serialize_param_list(&params)))?;

    // Evaluate Arguments
    let mut args = tail.iter().enumerate()
        .map(|(index, object)| eval(sym, env, object)
             // TODO this assumes param list always at position 1
             // Return index and error and process in caller
             .map_err(|e| e.trace(index + 1)))
        .collect::<Result<Vec<LispObject>, EvalError>>()?;

    // Return Binding
    let symbols = params.0.clone().into_iter();
    if let Some(rest_sym) = params.1 {
        let rest_args = args.split_off(params.0.len());
        args.push(LispObject::List(rest_args));
        Ok(symbols
           .chain(iter::once(rest_sym))
           .zip(args.into_iter())
           .collect::<Vec<(Symbol, LispObject)>>())
    } else {
        Ok(symbols
           .zip(args.into_iter())
           .collect::<Vec<(Symbol, LispObject)>>())
    }
}

fn split_param_list(lst: &mut Vec<Symbol>, rest_index: Option<usize>)
                    -> Result<Option<Symbol>, EvalError> {
    match rest_index {
        None => Ok(None),
        Some(rest_index) => if rest_index == lst.len() - 2 {
            let rest = lst.split_off(rest_index)[1];
            Ok(Some(rest))
        } else {
            Err(EvalError::new("&rest must be second to last in parameter list".to_string())
                .trace(rest_index))
        }
    }
}

fn parse_param_list(symbols: &Symbols, lst: Vec<LispObject>) -> Result<ParamList, EvalError> {
    let mut params = as_symbols(&lst)
        .map_err(|(e, index)| e.trace(index))?;
    let rest_index = params.iter().enumerate()
        .find(|(_, sym)| **sym == symbols.sym_rest)
        .map(|(index, _)| index);
    let rest = split_param_list(&mut params, rest_index)?;
    Ok((params, rest))
}

fn eval_special_form(sym: &Symbols, env: &mut Env,
                     sf: SpecialForm, tail: &[LispObject])
                -> Result<LispObject, EvalError> {
    match sf {
        SpecialForm::Quote => {
            assert_args(Match::Exact, tail, 1, || "special form quote".to_string())?;
            Ok(tail[0].clone())
        }
        SpecialForm::Begin => {
            assert_args(Match::Min, tail, 1, || "special form begin".to_string())?;
            let result = tail.iter().enumerate()
                .map(|(index, object)| eval(sym, env, object)
                     .map_err(|e| e.trace(index + 1)))
                .collect::<Result<Vec<LispObject>, EvalError>>()?;
            Ok(result[result.len() -1].clone())
        }
        SpecialForm::Def => {
            assert_args(Match::Exact, tail, 2, || "special form def".to_string())?;
            match tail[0] {
                LispObject::Symbol(s) => {
                    let value = eval(sym, env, &tail[1])
                        .map_err(|e| e.trace(2))?;
                    env.global(s, value.clone());
                    Ok(value)
                },
                _ => Err(EvalError::new("special form def must have a symbol in 1st place"
                                        .to_string())
                         .trace(1))
            }
        },
        SpecialForm::Set => {
            assert_args(Match::Exact, tail, 2, || "special form set".to_string())?;
            match tail[0] {
                LispObject::Symbol(s) => {
                    let value = eval(sym, env, &tail[1])
                        .map_err(|e| e.trace(2))?;
                    env.set(s, value.clone());
                    Ok(value)
                },
                _ => Err(EvalError::new("special form set must have a symbol in 1st place"
                                        .to_string())
                         .trace(1))
            }
        },
        SpecialForm::Fn => {
            // TODO allow only lispobject instead of vec<lispobject> as second param
            assert_args(Match::Min, tail, 2, || "special form fn".to_string())?;
            let param_list = tail[0].as_list()
                .map_err(|e| e.trace(1))?;
            let params = parse_param_list(sym, param_list)
                .map_err(|e| e.trace(1))?;
            let body = tail[1..].iter().cloned().collect();
            Ok(LispObject::Lambda(params, body))
        },
        SpecialForm::If => {
            // TODO allow only lispobject instead of vec<lispobject> as second param
            assert_args(Match::Min, tail, 2, || "special form if".to_string())?;
            let predicate = eval(sym, env, &tail[0])
                .and_then(|object| object.as_bool())
                .map_err(|e| e.trace(1))?;
            if predicate {
                eval(sym, env, &tail[1])
                    .map_err(|e| e.trace(2))
            } else if tail.len() == 2 {
                Ok(LispObject::Bool(false))
            } else {
                let result = tail[2..].iter().enumerate()
                    .map(|(index, object)| eval(sym, env, object)
                         .map_err(|e| e.trace(3 + index)))
                    .collect::<Result<Vec<LispObject>, EvalError>>()?;
                Ok(result[result.len() -1].clone())
            }
        },
        SpecialForm::Let => {
            assert_args(Match::Min, tail, 2, || "special form let".to_string())?;
            // let bindings = form[1].as_list()
            //     .map_err(|e| e.trace(1))?;
            // let evaled_bindings = bindings.iter().enumerate()
            //     .map(|binding| {
            //         let binding = binding.as_list()
            //             .map_err(|e|.trace(index).trace(1))?;
            //         let s = binding[0].as_symbol()
            //             .map_err(|e| e.trace(0).trace(index).trace(1))?;
            //         let v = eval(sym, env, binding[1])
            //             .map_err(|e| e.trace(0).trace(index).trace(1))?;
            //         (s, v)
            //     })
            //     .collect<Result<Vec<Symbol, LispObject>>>();

            // let result = in_scope(env, |mut env| {
            //     evaled_bindings.for_each(|(sym, value)| env.set(*sym, value));

            // })

            Err(EvalError::new("TODO implement special form let".to_string()).trace(0))
        },
    }
}

fn eval_lambda(sym: &Symbols, env: &mut Env,
               params: ParamList, forms: Vec<LispObject>, tail: &[LispObject])
               -> Result<LispObject, EvalError> {
    let binding = bind_param_list(sym, env, &params, tail)?;

    let result = in_scope(env, Some(binding), |mut env| {
        forms.iter().enumerate()
            .map(|(index, object)| eval(sym, &mut env, object)
                 .map_err(|e| e.trace(index).frame(LispObject::List(forms.clone()))))
            .collect::<Result<Vec<LispObject>, EvalError>>()
    })?;

    Ok(result[result.len() -1].clone())
}

fn eval(sym: &Symbols, env: &mut Env, object: &LispObject)
        -> Result<LispObject, EvalError> {
    match object {
        LispObject::List(l)   => {
            if l.len() == 0 {
                return Err(EvalError::new("apply received empty form".to_string()))
            }

            let tail = &l[1..];
            let head = eval(sym, env, &l[0])
                .map_err(|e| e.trace(0))?;

            match head {
                LispObject::SpecialForm(sf)
                    => eval_special_form(sym, env, sf, tail),
                LispObject::Lambda(params, forms)
                    => eval_lambda(sym, env, params, forms, tail),
                LispObject::Native(params, func) => {
                    let args = bind_param_list(sym, env, &params, tail)?
                        .into_iter().map(|(_, arg)| arg)
                        .collect::<Vec<LispObject>>();
                    func(&args[..])
                }
                _ => Err(EvalError::new("apply only implemented for LispObject::Native".to_string()).trace(0))
            }
        },
        LispObject::Symbol(s) => match env.resolve(s) {
            Some(object) => Ok(object.clone()),
            None =>         Err(EvalError::new(format!("Unbound symbol '{}'",
                                                       sym.as_string(s).unwrap_or("~~uninterned~~"))))
        }
        LispObject::String(s) => Ok(LispObject::String(s.to_string())),
        LispObject::Number(n) => Ok(LispObject::Number(*n)),
        LispObject::Bool(b)   => Ok(LispObject::Bool(*b)),
        LispObject::Native((p, r), f) => Ok(LispObject::Native((p.clone(), *r), *f)),
        LispObject::Lambda(_, _)
            => Err(EvalError::new(
                format!("Unexpected lambda. This is probably an internal error."))),
        LispObject::SpecialForm(_)
            => Err(EvalError::new(
                format!("Unexpected special form. This is probably an internal error.")))
    }
}

fn eval_frame(sym: &Symbols, env: &mut Env, obj: LispObject) -> Result<LispObject, EvalError> {
    eval(sym, env, &obj).map_err(|e| e.frame(obj))
}

fn main() {
    let mut rl = Editor::<()>::new();
    let mut symbols = Symbols::new();
    let mut env = create_root(&mut symbols);
    let mut reader = Reader::new();

    loop {
        let reader_stack = reader.len();
        let prompt = match reader_stack {
            0 => "? ".to_string(),
            _ => format!("> {}", "  ".repeat(reader_stack)),
        };

        match rl.readline(&prompt[..]) {
            Ok(line) => {
                let mut prog: Vec<LispObject> = vec![];
                match reader.partial(&mut symbols, &mut prog, &line) {
                    Ok(()) => for obj in prog {
                        match eval_frame(&symbols, &mut env, obj) {
                            Ok(object) => println!("{}", symbols.serialize_object(&object)),
                            Err(e) => handle_eval_error(&symbols, e),
                        }
                    }
                    result @ _ => if let Err(e) = handle_read_error(&line, result) {
                        break Err(e.to_string());
                    }
                }

                if line.trim().len() > 0 {
                    rl.add_history_entry(line);
                }
            },
            Err(ReadlineError::Eof)         => break Ok(()),
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(e) => break Err(e.to_string()),
        }
    }.unwrap_or_else(|err| print_message(&err));
}
