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
}

impl Symbols {
    pub fn new() -> Symbols {
        Symbols {
            registry: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 0,
        }
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

    pub fn as_string(&self, sym: &Symbol) -> Option<&str> {
        self.reverse.get(sym).map(|s| &s[..])
    }

    fn form_to_string(&self, l: &Vec<LispObject>) -> String {
        l.iter()
            .map(|o| self.serialize_object(o))
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn params_to_string(&self, ps: &Vec<Symbol>) -> String {
        ps.iter()
            .map(|o| self.as_string(o).unwrap_or("~~uninterned~~"))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn serialize_object(&self, obj: &LispObject) -> String {
        match obj {
            LispObject::Symbol(s) => format!("{}", self.as_string(s)
                                             .unwrap_or("~~uninterned~~")),
            LispObject::List(l) => format!("({})", self.form_to_string(l)),
            LispObject::Lambda(ps, fs) =>
                format!("(fn ({}) {})",
                        self.params_to_string(ps),
                        self.form_to_string(fs)),

            _ => obj.to_string(),
        }
    }
}

pub struct Env<'a> {
    vars: HashMap<Symbol, LispObject>,
    parent: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    fn root() -> Env<'static> {
        Env {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn derive<'b>(&mut self) -> Env where 'a: 'b {
        Env {
            vars: HashMap::new(),
            parent: Some(self),
        }
    }

    // pub fn global(&mut self, key: Symbol, value: LispObject) {
    //     match self.parent {
    //         Some(scope) => scope.global(key, value),
    //         None => {
    //             self.vars.insert(key, value);
    //         },
    //     }
    // }

    pub fn set(&mut self, key: Symbol, value: LispObject) {
        self.vars.insert(key, value);
    }

    pub fn resolve(&self, key: &Symbol) -> Option<&LispObject> {
        match self.vars.get(key) {
            Some(value) => Some(value),
            None => match self.parent {
                Some(scope) => scope.resolve(key),
                None => None,
            }
        }
    }
}

fn set_native(env: &mut Env, key: Symbol, value: Native) {
    env.set(key, LispObject::Native(value));
}

fn set_special(sym: &mut Symbols, env: &mut Env, sf: SpecialForm) {
    env.set(sym.intern(&sf.to_string()),
            LispObject::SpecialForm(sf));
}

pub fn create_root(symbols: &mut Symbols) -> Env<'static> {
    let mut root = Env::root();
    set_special(symbols, &mut root, SpecialForm::Def);
    set_special(symbols, &mut root, SpecialForm::Fn);
    set_special(symbols, &mut root, SpecialForm::If);
    set_special(symbols, &mut root, SpecialForm::Let);
    set_native(&mut root, symbols.intern("+"), native::add);
    set_native(&mut root, symbols.intern("*"), native::multiply);
    set_native(&mut root, symbols.intern("-"), native::minus);
    set_native(&mut root, symbols.intern("="), native::equal);
    root
}
