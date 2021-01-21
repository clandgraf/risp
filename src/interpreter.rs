use dirs;
use rustyline::{error::ReadlineError, Editor};
use rustyline;
use std::iter;
use std::fs::File;
use std::io::{prelude::*, BufReader, ErrorKind};
use std::path::{PathBuf};

use crate::{
    lisp_object::{
        Symbol,
        ParamList,
        EvalError,
        LispObject,
        SpecialForm,
        SerializeSymbol,
    },
    lisp_object_util::{
        Match,
        assert_args,
        as_symbols,
    },
    reader::{Reader, ReadError},
    env::{Env, Symbols, create_root},
    err::{handle_eval_error, handle_read_error, print_message},
    exc
};

pub enum ExecError {
    Read(ReadError),
    Eval(EvalError),
}

pub struct FunctionDef<'a> {
    params: ParamList,
    forms: &'a [LispObject],
    is_macro: bool,
}

pub struct Interpreter {
    symbols: Symbols,
    env: Env,
}

fn is_not_found(e: &ReadlineError) -> bool {
    matches!(e, ReadlineError::Io(error) if error.kind() == ErrorKind::NotFound)
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut symbols = Symbols::new();
        let env = create_root(&mut symbols);

        Interpreter {
            symbols: symbols,
            env: env,
        }
    }

    pub fn read_file(&mut self, f: &str) -> Result<(), String> {
        let mut reader = Reader::new();
        let mut prog: Vec<LispObject> = vec![];

        let file = File::open(f).map_err(|e| e.to_string())?;
        let fin = BufReader::new(file);

        for line in fin.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let input = line.splitn(2, ';').next().unwrap();
            reader.partial(&mut self.symbols, &mut prog, &input)
                .or_else(|e| handle_read_error(&line, e))
                .map_err(|e| e.to_string())?;
        }

        for object in prog {
            if let Err(e) = self.eval(&object) {
                handle_eval_error(&self.symbols, e);
                return Err(format!("Evaluation of {} failed.", f));
            }
        }

        Ok(())
    }

    pub fn interactive(&mut self) {
        let mut rl = Editor::<()>::new();

        let mut history_file = dirs::home_dir().unwrap_or(PathBuf::from("."));
        history_file.push(".risp-history");
        rl.load_history(&history_file).unwrap_or_else(|e| {
            if !is_not_found(&e) {
                print_message(&e);
            }
        });

        let mut reader = Reader::new();

        loop {
            let reader_stack = reader.len();
            let prompt = match reader_stack {
                0 => "? ".to_string(),
                _ => format!("> {}", "  ".repeat(reader_stack)),
            };

            match rl.readline(&prompt[..]) {
                Ok(line) => {
                    let result = self.handle_line(&mut reader, &line);
                    let result = self.handle_exec_error(&line, result);
                    if result.is_err() {
                        break result;
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

        rl.save_history(&history_file)
            .unwrap_or_else(|err| print_message(&err));
    }

    fn handle_line(&mut self, reader: &mut Reader, line: &String)
                   -> Result<(), ExecError> {
        let mut prog: Vec<LispObject> = vec![];
        reader.partial(&mut self.symbols, &mut prog, line)
            .map_err(ExecError::Read)?;
        for obj in prog {
            let result = self.eval(&obj)
                .map_err(|e| ExecError::Eval(e.frame(obj, Some(":in:".to_string()))))?;
            println!("{}", self.symbols.serialize_object(&result));
        }
        Ok(())
    }

    pub fn handle_exec_error(&self, line: &String, e: Result<(), ExecError>)
                             -> Result<(), String> {
        match e {
            Err(ExecError::Eval(e)) => handle_eval_error(&self.symbols, e),
            Err(ExecError::Read(e)) => {
                if let Err(e) = handle_read_error(line, e) {
                    return Err(e.to_string())
                }
            },
            _ => (),
        }
        Ok(())
    }

    fn eval(&mut self, object: &LispObject) -> Result<LispObject, EvalError> {
        match object {
            LispObject::List(l) => {
                if l.len() == 0 {
                    return Err(exc::apply_empty())
                }

                let tail = &l[1..];
                let head = self.eval(&l[0])
                    .map_err(|e| e.trace(0))?;

                match head {
                    LispObject::SpecialForm(sf)
                        => self.eval_special_form(sf, tail),
                    LispObject::Native(params, func) => {
                        // TODO handle err_in_expansion
                        let args = self.bind_param_list(&params, tail, true)
                            .map_err(|(e, _)| e)?
                            .into_iter().map(|(_, arg)| arg)
                            .collect::<Vec<LispObject>>();
                        func(&args[..])
                    }
                    LispObject::List(lst) => {
                        self.eval_form(&lst, tail)
                            .map_err(|(e, err_in_expansion)|
                                 if err_in_expansion {
                                     e.def_frame(&self.symbols,
                                                 LispObject::List(lst),
                                                 l[0].as_symbol().ok())
                                      .trace(0)
                                 } else {
                                     e
                                 }
                            )
                    }
                    _ => Err(exc::apply_unimpl()
                             .def_frame(&self.symbols, head, l[0].as_symbol().ok())
                             .trace(0))
                }
            },
            LispObject::Symbol(s) => match self.env.resolve(s) {
                Some(object) => Ok(object.clone()),
                None => Err(exc::unbound_symbol(self.symbols.as_string(s)))
            }
            LispObject::String(s) => Ok(LispObject::String(s.to_string())),
            LispObject::Number(n) => Ok(LispObject::Number(*n)),
            LispObject::Bool(b)   => Ok(LispObject::Bool(*b)),
            LispObject::Native((p, r), f) => Ok(LispObject::Native((p.clone(), *r), *f)),
            LispObject::SpecialForm(_)
                => Err(exc::unexpected_special_form())
        }
    }

    fn eval_form(&mut self, lst: &[LispObject], tail: &[LispObject])
                 -> Result<LispObject, (EvalError, bool)> {
        let FunctionDef {params, forms, is_macro} = self.parse_function_def(lst)
            .map_err(|e| (e, true))?;

        let binding = self.bind_param_list(&params, tail, !is_macro)?;
        let result = self.eval_body(Some(binding), forms)
            .map_err(|(e, index)| (e.trace(index + 2), true))?;

        if is_macro {
            self.eval(&result)
                .map_err(|e| (e.frame(result, Some("~>".to_string())).trace(0), true))
        } else {
            Ok(result)
        }
    }

    fn eval_special_form(&mut self, sf: SpecialForm, tail: &[LispObject])
                         -> Result<LispObject, EvalError> {
        match sf {
            SpecialForm::Quote => {
                assert_args(Match::Exact, tail, 1, || "special form quote".to_string())?;
                Ok(tail[0].clone())
            }
            SpecialForm::Begin => {
                assert_args(Match::Min, tail, 1, || "special form begin".to_string())?;
                let result = tail.iter().enumerate()
                    .map(|(index, object)| self.eval(object)
                         .map_err(|e| e.trace(index + 1)))
                    .collect::<Result<Vec<LispObject>, EvalError>>()?;
                Ok(result[result.len() -1].clone())
            }
            SpecialForm::Def => {
                assert_args(Match::Exact, tail, 2, || "special form def".to_string())?;
                match tail[0] {
                    LispObject::Symbol(s) => {
                        let value = self.eval(&tail[1])
                            .map_err(|e| e.trace(2))?;
                        self.env.global(s, value.clone());
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
                        let value = self.eval(&tail[1])
                            .map_err(|e| e.trace(2))?;
                        self.env.set(s, value.clone());
                        Ok(value)
                    },
                    _ => Err(EvalError::new("special form set must have a symbol in 1st place"
                                            .to_string())
                             .trace(1))
                }
            },
            SpecialForm::If => {
                // TODO allow only lispobject instead of vec<lispobject> as second param
                assert_args(Match::Min, tail, 2, || "special form if".to_string())?;
                let predicate = self.eval(&tail[0])
                    .and_then(|object| object.as_bool())
                    .map_err(|e| e.trace(1))?;
                if predicate {
                    self.eval(&tail[1])
                        .map_err(|e| e.trace(2))
                } else if tail.len() == 2 {
                    Ok(LispObject::Bool(false))
                } else {
                    let result = tail[2..].iter().enumerate()
                        .map(|(index, object)| self.eval(object)
                             .map_err(|e| e.trace(3 + index)))
                        .collect::<Result<Vec<LispObject>, EvalError>>()?;
                    Ok(result[result.len() -1].clone())
                }
            },
            SpecialForm::Let => {
                assert_args(Match::Min, tail, 2, || "special form let".to_string())?;
                let binding_forms = tail[0].as_list()
                    .map_err(|e| e.trace(1))?;

                let binding = binding_forms.iter().enumerate()
                    .map(|(index, b)| {
                        let b = b.as_list()
                            .map_err(|e| e.trace(index).trace(1))?;
                        let s = b[0].as_symbol()
                            .map_err(|e| e.trace(0).trace(index).trace(1))?;
                        let v = self.eval(&b[1])
                            .map_err(|e| e.trace(1).trace(index).trace(1))?;
                        Ok((s, v))
                    })
                    .collect::<Result<Vec<(Symbol, LispObject)>, EvalError>>()?;

                let forms = &tail[1..];
                self.eval_body(Some(binding), forms)
                    .map_err(|(err, index)| err.trace(index + 2))
            },
        }
    }

    fn eval_body(&mut self, binding: Option<Vec<(Symbol,LispObject)>>, forms: &[LispObject])
                 -> Result<LispObject, (EvalError, usize)> {
        self.env.push_scope();
        binding.map_or(
            (), |b| b.into_iter().for_each(
                |(sym, value)| self.env.set(sym, value)));

        let result = forms.iter().enumerate()
            .map(|(index, object)| self.eval(object)
                 .map_err(|e| (e, index)))
            .collect::<Result<Vec<LispObject>, (EvalError, usize)>>()
            .map(|mut v| v.pop().unwrap_or_else(|| LispObject::List(vec![])));

        self.env.pop_scope();
        result
    }

    fn parse_function_def<'a>(&mut self, lst: &'a [LispObject])
                              -> Result<FunctionDef<'a>, EvalError> {
        assert_args(Match::Min, &lst, 2, || "fn definition".to_string())?;

        let is_macro = match lst[0] {
            LispObject::Symbol(x) if x == self.symbols.sym_fn =>
                Ok(false),
            LispObject::Symbol(x) if x == self.symbols.sym_macro =>
                Ok(true),
            _ => Err(EvalError::new(format!("Expected `fn` or `macro` symbol, got `{}`",
                                            self.symbols.serialize_object(&lst[0])))
                     .trace(0))
        }?;

        // TODO mention param-list in err message
        let param_list = lst[1].as_list()
            .map_err(|e| e.trace(1))?;
        let params = self.parse_param_list(param_list)
            .map_err(|e| e.trace(1))?;
        let forms = &lst[2..];

        Ok(FunctionDef {
            params: params,
            forms: forms,
            is_macro: is_macro,
        })
    }

    fn parse_param_list(&mut self, lst: Vec<LispObject>) -> Result<ParamList, EvalError> {
        let mut params = as_symbols(&lst)
            .map_err(|(e, index)| e.trace(index))?;
        let rest_index = params.iter().enumerate()
            .find(|(_, sym)| **sym == self.symbols.sym_rest)
            .map(|(index, _)| index);
        let rest = split_param_list(&mut params, rest_index)?;
        Ok((params, rest))
    }

    fn bind_param_list<'a>(&mut self, params: &ParamList, tail: &[LispObject], eval_args: bool)
                   -> Result<Vec<(Symbol, LispObject)>, (EvalError, bool)> {
        // Check Validity of Arguments
        let m = match params.1 {
            None    => Match::Exact,
            Some(_) => Match::Min,
        };
        assert_args(m, tail, params.0.len(),
                    || format!("param list {}", self.symbols.serialize_param_list(&params)))
            .map_err(|e| (e.trace(1), true))?;

        // Evaluate Arguments
        let mut args = if eval_args {
            tail.iter().enumerate()
                .map(|(index, object)| self.eval(object)
                     // TODO this assumes param list always at position 1
                     // Return index and error and process in caller
                     .map_err(|e| e.trace(index + 1)))
                .collect::<Result<Vec<LispObject>, EvalError>>()
                .map_err(|e| (e, false))?
        } else {
            tail.to_vec()
        };

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
