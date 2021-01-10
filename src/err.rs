use ansi_term::Colour::{Blue, Red};
use std::fmt;
use crate::{
    env::Symbols,
    reader::ReadError,
    lisp_object::{LispObject, EvalError},
};

fn print_underline(start: usize, end: usize) {
    eprintln!(" {} {}{}",
              Blue.paint("|"), " ".repeat(start),
              Red.paint("^".repeat(end - start)));
}

fn print_range(input: &String, start: usize, end: usize) {
    eprintln!(" {} {}", Blue.paint("|"), input);
    print_underline(start, end);
}

pub fn print_message(displayable: &dyn fmt::Display) {
    eprintln!("{}: {}", Red.paint("Error"), displayable);
}

pub fn handle_read_error(input: &String, e: ReadError) -> Result<(), ReadError> {
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
    Ok(())
}

fn handle_failed_form(sym: &Symbols, form: &LispObject, stack: &[usize])
                      -> (String, usize, usize) {
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

pub fn handle_eval_error(sym: &Symbols, error: EvalError) {
    print_message(&error);
    for (form, trace) in error.frames {
        let (string, start, end) = handle_failed_form(sym, &form, &trace);
        print_range(&string, start, end);
    }
}
