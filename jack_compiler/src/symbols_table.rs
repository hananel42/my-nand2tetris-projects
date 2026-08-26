use crate::tokenizer::{Keyword, Token};
use std::collections::HashMap;
use crate::compilation_engine::{CompilationError, CompilationErrorType};
use crate::tokenizer::Token::Identifier;

#[derive(Copy, Clone)]
pub enum VarKind {
    Static,
    Field,
    Arg,
    Var,
}

#[derive(Clone)]
pub enum VarType {
    Int,
    Char,
    Boolean,
    Class(String),
}

pub struct Variable {
    var_type: VarType,
    kind: VarKind,
    index: usize,
}

struct SymbolTable {
    table: HashMap<String, Variable>,
    kind_count: [usize; 4], // Static, Field, Arg, Var
}
impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
            kind_count: [0; 4],
        }
    }

    fn define(&mut self, name: String, var_type: VarType, kind: VarKind) {
        let index = self.kind_count[kind as usize];
        self.kind_count[kind as usize] += 1;
        let variable = Variable {
            var_type,
            kind,
            index,
        };
        self.table.insert(name, variable);
    }

    fn var_count(&self, kind: VarKind) -> usize {
        self.kind_count[kind as usize]
    }

    fn get(&self, name: &str) -> Option<&Variable> {
        self.table.get(name)
    }
}

pub struct JackVariablesEngine {
    class_symbol_table: SymbolTable,
    subroutine_symbol_table: Option<SymbolTable>,
}
impl Variable {
    pub fn type_(&self) -> &VarType {
        &self.var_type
    }
    pub fn kind(&self) -> VarKind {
        self.kind
    }
    pub fn index(&self) -> u16 {
        self.index as u16
    }
}

impl TryFrom<Token> for VarKind {
    type Error = ();
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Keyword(k) => match k {
                Keyword::Static => Ok(VarKind::Static),
                Keyword::Field => Ok(VarKind::Field),
                Keyword::Var => Ok(VarKind::Var),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

impl TryFrom<Token> for VarType {
    type Error = ();
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Keyword(k) => match k {
                Keyword::Int => Ok(VarType::Int),
                Keyword::Char => Ok(VarType::Char),
                Keyword::Boolean => Ok(VarType::Boolean),
                _ => Err(()),
            },
            Token::Identifier(name) => Ok(VarType::Class(name)),
            _ => Err(()),
        }
    }
}

// A simple engine to handle the variables in a Jack program. It has two symbol tables, one for the class level and one for the subroutine level. The class level symbol table is used to store static and field variables, while the subroutine level symbol table is used to store arg and var variables.
// The engine can define new variables, get existing variables, and count the number of variables of a certain kind.
// Assumes that the engine is in a subroutine when defining an arg/lcl variable, (entered via enter_subroutine)
// and that the static/field variables definition is done when the engine isn't in a subroutine
impl JackVariablesEngine {
    pub fn new() -> JackVariablesEngine {
        JackVariablesEngine {
            class_symbol_table: SymbolTable::new(),
            subroutine_symbol_table: None,
        }
    }
    pub fn define(&mut self, name: String, type_: VarType, kind: VarKind) {
        if let Some(ref mut table) = self.subroutine_symbol_table {
            table.define(name, type_, kind);
        } else {
            self.class_symbol_table.define(name, type_, kind);
        }
    }
    pub fn get(&self, name: &str) -> Option<&Variable> {
        if let Some(ref table) = self.subroutine_symbol_table {
            if let Some(var) = table.get(name) {
                return Some(var);
            }
        }
        self.class_symbol_table.get(name)
    }


    pub fn var_count(&self, kind: VarKind) -> usize {
        let mut count = self.class_symbol_table.var_count(kind);
        if let Some(ref table) = self.subroutine_symbol_table {
            count += table.var_count(kind);
        }
        count
    }
    pub fn enter_subroutine(&mut self) {
        self.subroutine_symbol_table = Some(SymbolTable::new());
    }
    pub fn exit_subroutine(&mut self) {
        self.subroutine_symbol_table = None;
    }
}


