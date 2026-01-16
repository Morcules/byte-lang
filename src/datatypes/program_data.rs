use std::collections::HashMap;

use crate::datatypes::{ast_statements::{Function, FunctionArg, MemoryLocationsAst, Statement}, stack_frame::{StackFrame, StackVariable}, token::Token};

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
    pub stack_frames : Vec<StackFrame>,
    pub functions : HashMap<String, Function>,
    pub statements : Vec<Statement>,
    pub source_code : String,
    pub tokens : Vec<Token>,
    pub errors : Vec<String>
}

impl ProgramData {
    pub fn new() -> Self {
        Self { stack_frames: Vec::new(), functions: HashMap::new(), source_code: String::new(), tokens: Vec::new(), statements: Vec::new(), errors: Vec::new() }
    }

    pub fn get_stack_frame_by_index(&self, index : usize) -> &'_ StackFrame {
        return self.stack_frames.get(index).unwrap();
    }

    pub fn get_stack_variable_ref(&mut self, stack_frame : usize, var_name : &str) -> Option<StackVariableRef> {
        let frame = self.get_stack_frame_by_index(stack_frame);

        if let Some(var) = frame.variables.get(var_name) {
            let mut offset = 0;
            
            if frame.parent != usize::MAX {
                let mut current_frame_idx = frame.parent;
                loop {
                    let stack_frame_borrow = self.get_stack_frame_by_index(current_frame_idx);

                    offset += stack_frame_borrow.stack_mem_allocated;
                    
                    if stack_frame_borrow.parent == usize::MAX {
                        break;
                    }

                    current_frame_idx = stack_frame_borrow.parent;
                }
            }

            let func = self.functions.get(&frame.function).unwrap();

            let final_offset = func.stack_mem_allocated - (offset + var.offset + var.variable_size);
            
            return Some(StackVariableRef {
                local_offset: final_offset,
                var: var.clone(),
            });
        }

        if frame.parent == usize::MAX {
            return None;
        }

        self.get_stack_variable_ref(
            frame.parent,
            var_name
        )
    }

    pub fn get_function_stack_arg_ref(&self, stack_frame : usize, identifier : &str) -> Option<FunctionStackArgRef> {
        let function_name = self.get_stack_frame_by_index(stack_frame).function.clone();

        let func_mem_allocated = self.functions.get(&function_name).unwrap().arg_stack_mem_allocated.clone();
        let func_arg = self.functions.get(&function_name).unwrap().args.iter().find(|arg| arg.arg_name == identifier);

        let mut current_stack_frame = stack_frame;

        loop {
            let current_stack_frame_borrow = self.get_stack_frame_by_index(current_stack_frame);

            if current_stack_frame_borrow.parent == usize::MAX {
                break;
            }

            current_stack_frame = current_stack_frame_borrow.parent;
        }

        if let Some(func_arg_unwrapped) = func_arg {
            if let MemoryLocationsAst::Stack(stack_offset) = func_arg_unwrapped.memory_location {
                return Some(FunctionStackArgRef{local_offset: func_mem_allocated - stack_offset - func_arg_unwrapped.arg_var_type.get_variable_size(), var: func_arg_unwrapped.clone()});
            }

            return None;
        } else {
            return None;
        }
    }

    // Get total amount of mem to allocate (MAX used by scopes)
    pub fn traverse_stack_frame_memory(&self, stack_frame_idx: usize) -> usize {
        let frame = self.get_stack_frame_by_index(stack_frame_idx);

        let mut required = 0;

        for child_idx in &frame.children {
            let child_needed = self.traverse_stack_frame_memory(child_idx.clone());
            if child_needed > required {
                required = child_needed;
            }
        }

        return required + frame.stack_mem_allocated;
    }

    // Get stack memory allocated for function
    pub fn get_func_stack_memory(&self, stack_frame_idx: usize) -> usize {
        let stack_frame_borrow = self.get_stack_frame_by_index(stack_frame_idx);
        let func = self.functions.get(&stack_frame_borrow.function).unwrap();
        let mem = func.stack_mem_allocated;

        return mem;
    }
}
