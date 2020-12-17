use ansi_term::Colour::{Blue, Red};
use rustyline::{error::ReadlineError, Editor};
use rustyline;
use std::fmt;

use lisp::{
    reader::{Reader, ReadError},
    LispObject,
    EvalError,
    Env,
    as_symbol,
    create_root};

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

fn handle_failed_form(form: &LispObject, stack: &[usize]) -> (String, usize, usize) {
    if stack.len() == 0 {
        let string = form.to_string();
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
                        let (s, off0, off1) = handle_failed_form(object, &stack[0..stack_len]);
                        start = off0 + string.len();
                        end = off1 + string.len();
                        string.push_str(&s);
                    } else {
                        string.push_str(&object.to_string());
                    }
                    if index == l.len() - 1 {
                        string.push_str(")");
                    } else {
                        string.push_str(" ");
                    }
                }
                (string, start, end)
            },
            _ => handle_failed_form(form, &stack[0..0])
        }
    }
}

fn handle_eval_error(form: &LispObject, error: EvalError) {
    let (string, start, end) = handle_failed_form(form, &error.trace[..]);
    print_range_error(&string, &error, start, end);
}

fn apply(env: &mut Env, form: &Vec<LispObject>) -> Result<LispObject, EvalError> {
    if form.len() == 0 {
        return Err(EvalError::new("apply received empty form".to_string()))
    }

    let args: Vec<LispObject> = form[1..].iter().enumerate()
        .map(|(index, object)| eval(env, object)
             .map_err(|e| e.trace(index + 1)))
        .collect::<Result<Vec<LispObject>, EvalError>>()?;

    let head: &LispObject = eval(env, &form[0])
        .map_err(|e| e.trace(0))
        .and_then(|sym| as_symbol(&sym)
                  .map_err(|e| e.trace(0)))
        .and_then(|sym| env.resolve(&sym[..])
                  .map_or_else(|| Err(EvalError::new(format!("Unbound symbol '{}'", sym)).trace(0)),
                               |head| Ok(head)))?;

    match head {
        LispObject::Native(f) => f(&args[..]),
        _ => Err(EvalError::new("apply only implemented for LispObject::Native".to_string()).trace(0))
    }
}

fn eval(env: &mut Env, object: &LispObject) -> Result<LispObject, EvalError> {
    match object {
        LispObject::List(l)   => apply(env, &l),
        LispObject::Symbol(s) => Ok(LispObject::Symbol(s.to_string())),
        LispObject::String(s) => Ok(LispObject::String(s.to_string())),
        LispObject::Number(n) => Ok(LispObject::Number(*n)),
        LispObject::Native(f) => Ok(LispObject::Native(*f)),
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    let mut env: Env = create_root();
    let mut reader: Reader = Reader::new();

    loop {
        let reader_stack = reader.len();
        let prompt = match reader_stack {
            0 => "? ".to_string(),
            _ => format!("> {}", "  ".repeat(reader_stack)),
        };

        match rl.readline(&prompt[..]) {
            Ok(line) => {
                let mut prog: Vec<LispObject> = vec![];
                let res = reader.partial(&mut prog, &line);
                if let Err(e) = handle_read_error(&line, res) {
                    break Err(e.to_string());
                } else {
                    for obj in prog {
                        match eval(&mut env, &obj) {
                            Ok(object) => println!("{}", &object),
                            Err(e) => handle_eval_error(&obj, e),
                        }
                    }
                }
            },
            Err(ReadlineError::Eof)         => break Ok(()),
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(e) => break Err(e.to_string()),
        }
    }.unwrap_or_else(|err| print_error_msg(&err));
}
