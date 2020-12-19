use ansi_term::Colour::{Blue, Red};
use rustyline::{error::ReadlineError, Editor};
use rustyline;
use std::fmt;

use lisp::{
    lisp_object::{
        EvalError,
        LispObject,
        SpecialForm,
    },
    lisp_object_util::{
        assert_exact_form_args,
        assert_min_form_args,
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

fn print_range_error(input: &String, error: &dyn fmt::Display, start: usize, end: usize) {
    print_error_msg(&error);
    eprintln!(" {} {}", Blue.paint("|"), input);
    print_underline(start, end);
}

fn print_error_msg(displayable: &dyn fmt::Display) {
    eprintln!("{}: {}", Red.paint("Error"), displayable);
}

fn handle_read_error(input: &String, result: Result<(), ReadError>) -> Result<(), ReadError> {
    if let Err(e) = result {
        match e {
            ReadError::UnknownCharacter((start, end)) => print_range_error(input, &e, start, end),
            ReadError::UnexpectedRbrace((start, end)) => print_range_error(input, &e, start, end),
            ReadError::UnexpectedEndOfString => print_error_msg(&ReadError::UnexpectedEndOfString),
            ReadError::InternalError => return Err(ReadError::InternalError),
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

fn handle_eval_error(sym: &Symbols, form: &LispObject, error: EvalError) {
    let (string, start, end) = handle_failed_form(sym, form, &error.trace[..]);
    print_range_error(&string, &error, start, end);
}

fn apply_eval_args(sym: &Symbols, env: &mut Env, form: &Vec<LispObject>)
                   -> Result<Vec<LispObject>, EvalError> {
    form[1..].iter().enumerate()
        .map(|(index, object)| eval(sym, env, object)
             .map_err(|e| e.trace(index + 1)))
        .collect::<Result<Vec<LispObject>, EvalError>>()
}

fn apply(sym: &Symbols, env: &mut Env, form: &Vec<LispObject>) -> Result<LispObject, EvalError> {
    if form.len() == 0 {
        return Err(EvalError::new("apply received empty form".to_string()))
    }

    let head = eval(sym, env, &form[0])
        .map_err(|e| e.trace(0))?;

    match head {
        LispObject::SpecialForm(SpecialForm::Def) => {
            assert_exact_form_args(&form[1..], 2, || "special form def".to_string())?;
            match form[1] {
                LispObject::Symbol(s) => {
                    let value = eval(sym, env, &form[2])
                        .map_err(|e| e.trace(2))?;
                    env.set(s, value.clone());
                    Ok(value)
                },
                _ => Err(EvalError::new("special form def must have a symbol in 1st place"
                                        .to_string())
                         .trace(1))
            }
        },
        LispObject::SpecialForm(SpecialForm::Fn) => {
            assert_min_form_args(&form[1..], 2, || "special form fn".to_string())?;
            let plist = &form[1].as_list()
                .map_err(|e| e.trace(1))?;
            let params = as_symbols(plist)
                .map_err(|(e, index)| e.trace(index).trace(1))?;
            let body = form[2..].iter().cloned().collect();
            Ok(LispObject::Lambda(params, body))
        },
        LispObject::SpecialForm(SpecialForm::If)
            => {
                assert_min_form_args(&form[1..], 2, || "special form if".to_string())?;
                let predicate = eval(sym, env, &form[1])
                    .and_then(|object| object.as_bool())
                    .map_err(|e| e.trace(1))?;
                if predicate {
                    eval(sym, env, &form[2])
                        .map_err(|e| e.trace(2))
                } else if form.len() == 3 {
                    Ok(LispObject::Bool(false))
                } else {
                    let result = form[3..].iter().enumerate()
                        .map(|(index, object)| eval(sym, env, object)
                             .map_err(|e| e.trace(3 + index)))
                        .collect::<Result<Vec<LispObject>, EvalError>>()?;
                    Ok(result[result.len() -1].clone())
                }
            },
        LispObject::SpecialForm(SpecialForm::Let)
            => {
                // TODO set trace correctly
                assert_min_form_args(&form[1..], 2, || "special form let".to_string())?;
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

                // let mut env = env.derive();
                // evaled_bindings.for_each(|(sym, value)| env.set(*sym, value));

                Err(EvalError::new("TODO implement special form let".to_string()).trace(0))
            },
        LispObject::Lambda(params, forms) => {
            assert_exact_form_args(&form[1..], params.len(), || "lambda".to_string())?;
            let args = apply_eval_args(sym, env, form)?;
            let mut env = env.derive();
            params.iter().zip(args)
                .for_each(|(sym, value)| env.set(*sym, value));
            // TODO need a new entry for stack trace, as we're
            //      descending into a new set of forms here
            let result = forms.iter().enumerate()
                .map(|(index, object)| eval(sym, &mut env, object)
                     .map_err(|e| EvalError::new(e.message)))
                .collect::<Result<Vec<LispObject>, EvalError>>()?;
            Ok(result[result.len() -1].clone())
        }
        LispObject::Native(f) => {
            let args = apply_eval_args(sym, env, form)?;
            f(&args[..])
        }
        _ => Err(EvalError::new("apply only implemented for LispObject::Native".to_string()).trace(0))
    }
}

fn eval(sym: &Symbols, env: &mut Env, object: &LispObject) -> Result<LispObject, EvalError> {
    match object {
        LispObject::List(l)   => apply(sym, env, &l),
        LispObject::Symbol(s) => match env.resolve(s) {
            Some(object) => Ok(object.clone()),
            None =>         Err(EvalError::new(format!("Unbound symbol '{}'",
                                                       sym.as_string(s).unwrap_or("~~uninterned~~"))))
        }
        LispObject::String(s) => Ok(LispObject::String(s.to_string())),
        LispObject::Number(n) => Ok(LispObject::Number(*n)),
        LispObject::Bool(b)   => Ok(LispObject::Bool(*b)),
        LispObject::Native(f) => Ok(LispObject::Native(*f)),
        LispObject::Lambda(_, _)
            => Err(EvalError::new(
                format!("Unexpected lambda. This is probably an internal error."))),
        LispObject::SpecialForm(_)
            => Err(EvalError::new(
                format!("Unexpected special form. This is probably an internal error.")))
    }
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
                        match eval(&symbols, &mut env, &obj) {
                            Ok(object) => println!("{}", symbols.serialize_object(&object)),
                            Err(e) => handle_eval_error(&symbols, &obj, e),
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
    }.unwrap_or_else(|err| print_error_msg(&err));
}
