use clap::{Arg, App};
use lisp::interpreter::Interpreter;

fn main() {
    let matches = App::new("lisp")
        .version("0.1.0")
        .author("Christoph Landgraf <christoph.landgraf@googlemail.com>")
        .about("Simple Lisp Interpreter in Rust")
        .arg(Arg::with_name("interactive")
             .short("i")
             .long("interactive")
             .help("If a file is provided, go to interpreter after that."))
        .arg(Arg::with_name("file")
             .help("If provided run the file.")
             .index(1))
        .get_matches();

    let mut interpreter = Interpreter::new();
    if let Some(f) = matches.value_of("file") {
        if let Err(e) = interpreter.read_file(&f) {
            println!("{}:", e);
            return;
        }
    }
    interpreter.interactive();
}
