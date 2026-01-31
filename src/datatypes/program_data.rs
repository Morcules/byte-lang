use std::collections::HashMap;

use crate::datatypes::{ast_statements::{Function, FunctionArg, MemoryLocationsAst, Statement}, scope::{Scope, StackVariable}, token::Token};

#[derive(Clone, Debug, PartialEq)]
pub struct StackVariableRef {
    pub local_offset : usize,
    pub var: StackVariable
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionStackArgRef {
    pub local_offset : usize,
    pub var: FunctionArg
}

#[derive(PartialEq, Clone, Debug)]
pub struct ProgramData {
    pub scopes : Vec<Scope>,
    pub functions : HashMap<String, Function>,
    pub statements : Vec<Statement>,
    pub source_codes : HashMap<String, String>,
    pub tokens : Vec<Token>,
    pub errors : Vec<String>
}

impl ProgramData {
    pub fn new() -> Self {
        Self { scopes: Vec::new(), functions: HashMap::new(), source_codes: HashMap::new(), tokens: Vec::new(), statements: Vec::new(), errors: Vec::new() }
    }

    pub fn get_scope_by_index(&self, index : usize) -> &'_ Scope {
        return self.scopes.get(index).unwrap();
    }

    pub fn get_scope_by_index_mut(&mut self, index : usize) -> &'_ mut Scope {
        return self.scopes.get_mut(index).unwrap();
    }

    pub fn get_stack_variable_ref(&mut self, scope : usize, var_name : &str) -> Option<StackVariableRef> {
        let scope = self.get_scope_by_index(scope);

        if let Some(var) = scope.variables.get(var_name) {
            let mut offset = 0;
            
            if scope.parent != usize::MAX {
                let mut current_scope_idx = scope.parent;
                loop {
                    let scope_borrow = self.get_scope_by_index(current_scope_idx);

                    offset += scope_borrow.stack_mem_allocated;
                    
                    if scope_borrow.parent == usize::MAX {
                        break;
                    }

                    current_scope_idx = scope_borrow.parent;
                }
            }

            let func = self.functions.get(&scope.function).unwrap();

            let final_offset = func.stack_mem_allocated - (offset + var.offset + var.variable_size);
            
            return Some(StackVariableRef {
                local_offset: final_offset,
                var: var.clone(),
            });
        }

        if scope.parent == usize::MAX {
            return None;
        }

        self.get_stack_variable_ref(
            scope.parent,
            var_name
        )
    }

    // Get refrence to stack arg
    pub fn get_function_stack_arg_ref(&self, scope : usize, identifier : &str) -> Option<FunctionStackArgRef> {
        let function_name = self.get_scope_by_index(scope).function.clone();

        let func_arg_stack_mem_allocated = self.functions.get(&function_name).unwrap().arg_stack_mem_allocated.clone();
        let func_stack_mem_allocated = self.functions.get(&function_name).unwrap().stack_mem_allocated.clone();
        let func_arg = self.functions.get(&function_name).unwrap().args.iter().find(|arg| arg.arg_name == identifier);

        if let Some(func_arg_unwrapped) = func_arg {
            if let MemoryLocationsAst::Stack(stack_offset) = func_arg_unwrapped.memory_location {
                return Some(FunctionStackArgRef{local_offset: (func_arg_stack_mem_allocated + func_stack_mem_allocated + 16) - stack_offset - func_arg_unwrapped.arg_var_type.get_variable_size(), var: func_arg_unwrapped.clone()});
            }

            return None;
        } else {
            return None;
        }
    }

    // Get total amount of mem to allocate (MAX used by scopes)
    pub fn traverse_scope_memory(&self, scope_idx: usize) -> usize {
        let scope = self.get_scope_by_index(scope_idx);

        let mut required = 0;

        for child_idx in &scope.children {
            let child_needed = self.traverse_scope_memory(child_idx.clone());
            if child_needed > required {
                required = child_needed;
            }
        }

        return required + scope.stack_mem_allocated;
    }

    // Get stack memory allocated for function
    pub fn get_func_stack_memory(&self, scope_idx: usize) -> usize {
        let scope_borrow = self.get_scope_by_index(scope_idx);
        let func = self.functions.get(&scope_borrow.function).unwrap();
        let mem = func.stack_mem_allocated;

        return mem;
    }
}
