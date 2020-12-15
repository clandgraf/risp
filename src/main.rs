use rustyline::{error::ReadlineError, Editor};
use rustyline;

use lisp::{Parser, SExp};

fn main() {
    let mut rl = Editor::<()>::new();
    let mut prog: Vec<SExp> = vec![];
    let mut parser: Parser = Parser::new();

    loop {
        let prompt = match parser.len() {
            0 => "? ".to_string(),
            _ => format!("  {}", "> ".repeat(parser.len())),
        };

        match rl.readline(&prompt[..]) {
            Ok(line) => match parser.partial(&mut prog, &line) {
                Ok(()) => (),
                Err(e) => break Err(e.to_string()),
            },
            Err(ReadlineError::Eof)         => break Ok(()),
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(e) => break Err(e.to_string()),
        }
    }.unwrap_or_else(|err| eprintln!("Error in reader: {}", err));
}
