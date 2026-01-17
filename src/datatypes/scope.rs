use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::datatypes::ast_statements::{CgStatement, Statement, VariableType};

#[derive(Clone, Debug, PartialEq)]
pub struct StackVariable {
    pub variable_type : VariableType,
    pub variable_size : usize, 
    pub offset : usize 
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub variables: HashMap<String, StackVariable>,
    pub stack_mem_allocated : usize,
    pub statements : Vec<Statement>,
    pub cg_statements : Vec<CgStatement>,
    pub children : Vec<usize>,
    pub parent : usize,
    pub function : String
}

impl Scope {
    pub fn new(parent : usize, function_name : String) -> Self {
        return Self { variables: HashMap::new(), stack_mem_allocated: 0, statements: Vec::new(), cg_statements: Vec::new(), children: Vec::new(), parent: parent, function: function_name }
    }

    pub fn default(function_name : String) -> Self {
        Self { variables: HashMap::new(), stack_mem_allocated: 0, statements: Vec::new(), cg_statements: Vec::new(), children: Vec::new(), parent: usize::MAX, function: function_name  }
    }
}
