use std::collections::HashMap;
use std::collections::hash_map::Entry;

mod lexer;
pub mod lisp_object;
pub mod lisp_object_util;
mod native;
pub mod reader;

use crate::lisp_object::{
    LispObject,
    Native,
    SpecialForm,
    Symbol,
};

pub struct Symbols {
    registry: HashMap<String, Symbol>,
    reverse: HashMap<Symbol, String>,
    next_id: Symbol,

    pub sym_quote: Symbol,
    pub sym_quasiquote: Symbol,
    pub sym_unquote: Symbol,
    pub sym_unquote_splice: Symbol,
    pub sym_rest: Symbol,
}

impl Symbols {
    pub fn new() -> Symbols {
        let mut symbols = Symbols {
            registry: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 0,

            sym_quote: 0,
            sym_quasiquote: 0,
            sym_unquote: 0,
            sym_unquote_splice: 0,
            sym_rest: 0,
        };
        symbols.sym_quote = symbols.intern("quote");
        symbols.sym_quasiquote = symbols.intern("quasiquote");
        symbols.sym_unquote = symbols.intern("unquote");
        symbols.sym_unquote_splice = symbols.intern("unquote-splice");
        symbols.sym_rest = symbols.intern("&rest");
        symbols
    }

    pub fn intern(&mut self, name: &str) -> Symbol {
        match self.registry.entry(name.to_string()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(_) => {
                self.next_id += 1;
                self.registry.insert(name.to_string(), self.next_id);
                self.reverse.insert(self.next_id, name.to_string());
                self.next_id
            }
        }
    }

    pub fn symbol(&mut self, name: &str) -> LispObject {
        LispObject::Symbol(self.intern(name))
    }

    pub fn quote(&mut self, obj: LispObject) -> LispObject {
        LispObject::List(vec![LispObject::Symbol(self.sym_quote), obj])
    }

    pub fn quasi_quote(&mut self, obj: LispObject) -> LispObject {
        LispObject::List(vec![LispObject::Symbol(self.sym_quasiquote), obj])
    }

    pub fn unquote(&mut self, obj: LispObject) -> LispObject {
        LispObject::List(vec![LispObject::Symbol(self.sym_unquote), obj])
    }

    pub fn unquote_splice(&mut self, obj: LispObject) -> LispObject {
        LispObject::List(vec![LispObject::Symbol(self.sym_unquote_splice), obj])
    }

    pub fn as_string(&self, sym: &Symbol) -> Option<&str> {
        self.reverse.get(sym).map(|s| &s[..])
    }

    fn form_to_string(&self, l: &Vec<LispObject>) -> String {
        l.iter()
            .map(|o| self.serialize_object(o))
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn pos_params_to_string(&self, ps: &Vec<Symbol>) -> String {
        ps.iter()
            .map(|o| self.as_string(o).unwrap_or("~~uninterned~~"))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn rest_param_to_string(&self, sym: Option<Symbol>) -> String {
        match sym {
            Some(s) => format!(" &rest {}", self.as_string(&s)
                               .unwrap_or("~~uninterned~~")),
            None => "".to_string(),
        }
    }

    pub fn serialize_object(&self, obj: &LispObject) -> String {
        match obj {
            LispObject::Symbol(s) => format!("{}", self.as_string(s)
                                             .unwrap_or("~~uninterned~~")),
            LispObject::List(l) => format!("({})", self.form_to_string(l)),
            LispObject::Lambda(ps, fs) =>
                format!("(fn ({}{}) {})",
                        self.pos_params_to_string(&ps.0),
                        self.rest_param_to_string(ps.1),
                        self.form_to_string(fs)),
            LispObject::Bool(true) => "#t".to_string(),
            LispObject::Bool(false) => "#f".to_string(),
            LispObject::SpecialForm(sf) => format!("{}", sf),
            LispObject::String(s) => format!("\"{}\"", s),
            LispObject::Number(n) => format!("{}", n.to_string()),
            LispObject::Native(_) => "~~native~~".to_string(),
        }
    }
}

pub struct Env {
    vars: Vec<HashMap<Symbol, LispObject>>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            vars: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.vars.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.vars.pop();
    }

    pub fn set(&mut self, key: Symbol, value: LispObject) {
        self.vars.last_mut().and_then(|v| v.insert(key, value));
    }

    pub fn global(&mut self, key: Symbol, value: LispObject) {
        self.vars.first_mut().and_then(|v| v.insert(key, value));
    }

    pub fn resolve(&self, key: &Symbol) -> Option<&LispObject> {
        match self.vars.iter().rev()
            .find(|scope| scope.contains_key(key)) {
                Some(scope) => scope.get(key),
                None => None,

            }
    }
}

fn set_native(env: &mut Env, key: Symbol, value: Native) {
    env.global(key, LispObject::Native(value));
}

fn set_special(sym: &mut Symbols, env: &mut Env, sf: SpecialForm) {
    env.global(sym.intern(&sf.to_string()),
               LispObject::SpecialForm(sf));
}

pub fn create_root(symbols: &mut Symbols) -> Env {
    let mut root = Env::new();
    set_special(symbols, &mut root, SpecialForm::Def);
    set_special(symbols, &mut root, SpecialForm::Set);
    set_special(symbols, &mut root, SpecialForm::Fn);
    set_special(symbols, &mut root, SpecialForm::If);
    set_special(symbols, &mut root, SpecialForm::Let);
    set_special(symbols, &mut root, SpecialForm::Begin);
    set_special(symbols, &mut root, SpecialForm::Quote);
    set_native(&mut root, symbols.intern("+"), native::add);
    set_native(&mut root, symbols.intern("*"), native::multiply);
    set_native(&mut root, symbols.intern("-"), native::minus);
    set_native(&mut root, symbols.intern("="), native::equal);
    set_native(&mut root, symbols.intern("first"), native::first);
    set_native(&mut root, symbols.intern("rest"), native::rest);
    root
}
