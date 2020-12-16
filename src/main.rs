use ansi_term::Colour::{Blue, Red};
use rustyline::{error::ReadlineError, Editor};
use rustyline;
use std::fmt;

use lisp::{Reader, ReadError, SExp};

fn print_range_error(input: &String, err: ReadError, start: usize, end: usize) {
    print_error_msg(&err);
    eprintln!(" {} {}",   Blue.paint("|"), input);
    eprintln!(" {} {}{}", Blue.paint("|"), " ".repeat(start), Red.paint("^".repeat(end - start)));
}

fn print_error_msg(displayable: &dyn fmt::Display) {
    eprintln!("{}: {}", Red.paint("Error"), displayable);
}

fn handle_error(input: &String, err: ReadError) -> Result<(), ReadError> {
    match err {
        ReadError::UnknownCharacter((start, end)) => print_range_error(input, err, start, end),
        ReadError::UnexpectedRbrace((start, end)) => print_range_error(input, err, start, end),
        ReadError::UnexpectedEndOfString => print_error_msg(&ReadError::UnexpectedEndOfString),
        ReadError::InternalError => return Err(ReadError::InternalError),
    }
    Ok(())
}

fn main() {
    let mut rl = Editor::<()>::new();
    let mut prog: Vec<SExp> = vec![];
    let mut parser: Reader = Reader::new();

    loop {
        let prompt = match parser.len() {
            0 => "? ".to_string(),
            _ => format!("> {}", "  ".repeat(parser.len())),
        };

        match rl.readline(&prompt[..]) {
            Ok(line) =>  if let Err(e) = parser.partial(&mut prog, &line) {
                if let Err(e) = handle_error(&line, e) {
                    break Err(e.to_string());
                }
            },
            Err(ReadlineError::Eof)         => break Ok(()),
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(e) => break Err(e.to_string()),
        }
    }.unwrap_or_else(|err| print_error_msg(&err));
}
