use std::{collections::HashMap, fmt::format};

use crate::datatypes::{assembly_instructions::asm::*, ast_statements::{AstIdentifiers, CgBuiltInFunctions, CgExpression, CgIdentifiers, CgStatement, CgStatementType, Literal, MemoryLocationsAst, VariableType}, general_functions::align_memory, program_data::{ProgramData, StackVariableRef}, stack_frame::StackFrame};

pub struct CodeGenerator<'a> {
    program_data: &'a mut ProgramData,
    stack_ptr: usize,
    labels: HashMap<String, String>
}

impl<'a> CodeGenerator<'a> {
    pub fn new(program_data : &'a mut ProgramData) -> Self {
        return Self{program_data, stack_ptr: 0, labels: HashMap::new()};
    }

    pub fn process_statement(&mut self, statement : &CgStatement, label: &str, stack_frame : usize) -> () {
        match statement.statement_type.clone() {
            CgStatementType::VariableAssignment(var_init) => {
                let compiled_code = &self.init_var(var_init.stack_offset, var_init.variable_type.clone(), var_init.assign_value);

                self.push_compiled_code_to_label(label, compiled_code);
            },
            CgStatementType::BuiltInFunction(built_in_function) => {
                match built_in_function {
                    CgBuiltInFunctions::BranchLinked(branch_linked) => {
                        let mut result = String::new();

                        let function_args = self.program_data.functions.get(&branch_linked.function_name).unwrap().args.clone();
                        let function_stack_args_mem_allocated = self.program_data.functions.get(&branch_linked.function_name).unwrap().stack_mem_allocated;

                        if function_stack_args_mem_allocated != 0 {
                            result.push_str(&allocate_stack_memory(function_stack_args_mem_allocated));
                        }

                        for i in 0..function_args.len() {
                            let arg_provided = branch_linked.args.get(i).unwrap();
                            let arg_expecting = function_args.get(i).unwrap();

                            match arg_expecting.memory_location.clone() {
                                MemoryLocationsAst::Register(register) => {
                                    match arg_provided {
                                        CgExpression::Literal(Literal::Number(num)) => {
                                            result.push_str(&mov_num_to_reg(&register, num.clone()));
                                        },
                                        CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data)) => {
                                            result.push_str(&variable_to_reg(&register, function_stack_args_mem_allocated + stack_var_data.offset, stack_var_data.variable_type.clone()));
                                        },
                                        _ => todo!()
                                    }
                                },
                                MemoryLocationsAst::Stack(stack_arg_offset) => {
                                    match arg_provided {
                                        CgExpression::Literal(Literal::Number(num)) => {
                                            let var_size = arg_expecting.arg_var_type.get_variable_size();

                                            result.push_str(&store_literal_to_stack(arg_expecting.arg_var_type.clone(), num.clone(), function_stack_args_mem_allocated - stack_arg_offset - var_size));
                                        },
                                        CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data)) => {
                                            let mut stack_var_data_clone = stack_var_data.clone();

                                            let var_size = stack_var_data_clone.variable_type.get_variable_size();

                                            stack_var_data_clone.offset += function_stack_args_mem_allocated;

                                            result.push_str(&self.init_var(function_stack_args_mem_allocated - stack_arg_offset - var_size, stack_var_data_clone.variable_type.clone(), CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data_clone))));
                                        },
                                        _ => unreachable!()
                                    }
                                }
                                _ => todo!()
                            }
                        }

                        result.push_str(&jump_to_function(&branch_linked.function_name));

                        if function_stack_args_mem_allocated != 0 {
                            result.push_str(&deallocate_stack_memory(function_stack_args_mem_allocated));
                        }

                        self.push_compiled_code_to_label(label, &result);
                    },
                    CgBuiltInFunctions::Assembly(assembly_code) => {
                        self.push_compiled_code_to_label(label, &assembly_code);
                    }
                }
            }
        };
    }

    pub fn init_var(&mut self, target_offset : usize, variable_type : VariableType, expression : CgExpression) -> String {
        match (variable_type.clone(), expression.clone()) {
            (
                _,
                CgExpression::Literal(Literal::Number(num))
            ) => {
                return String::from(store_literal_to_stack(variable_type, num, target_offset));
            },
            (
                _,
                CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data))
            ) => {
                return String::from(format!("{}{}", variable_to_reg(&temp_reg_for_type(variable_type.clone(), true), stack_var_data.offset, variable_type.clone()), store_reg_to_stack(&temp_reg_for_type(variable_type.clone(), false), target_offset, variable_type)));
            }
            _ => unreachable!()
        }
    }

    pub fn initialize_stack_frame(&mut self, label: &str, stack_frame : usize) -> () {
        let mem = self.get_stack_frame_by_index(stack_frame).stack_mem_allocated.clone();

        self.push_compiled_code_to_label(label, &create_stack_frame(mem));
    }

    pub fn return_stack_frame(&mut self, label: &str, stack_frame : usize) -> () {
        let stack_frame_borrow = self.get_stack_frame_by_index(stack_frame);

        self.push_compiled_code_to_label(label, &destroy_stack_frame(stack_frame_borrow.stack_mem_allocated));

        return;
    }

    pub fn push_compiled_code_to_label(&mut self, label: &str, code: &str) -> () {
        self.labels.get_mut(label).unwrap().push_str(code);
    }

    pub fn process_stack_frame(&mut self, stack_frame : usize, label: &str) -> () {
        self.initialize_stack_frame(label, stack_frame);

        for statement in self.get_stack_frame_by_index(stack_frame).cg_statements.clone().iter() {
            self.process_statement(statement, label, stack_frame);
        }

        self.return_stack_frame(label, stack_frame);

        return;
    }

    pub fn process_stack_frame_and_children(&mut self, stack_frame_index : usize, func_name : &str) -> () {
        self.traverse_stack_frame_children(stack_frame_index, func_name);
    }

    pub fn process_labels(&mut self) -> String {
        println!("Labels: {:?}", self.labels);

        let mut result = String::new();

        for (label, code) in self.labels.iter() {
            result.push_str(&format!("_{}:\n{}\n", label, code));
        }

        return result;
    }

    pub fn process_all_functions(&mut self) -> () {
        for (function_name, stack_frame) in self.program_data.functions.clone() {
            self.labels.insert(function_name.clone(), String::new());

            self.process_stack_frame_and_children(stack_frame.first_stack_frame.clone(), &function_name);
        }

        return;
    }

    pub fn traverse_stack_frame_children(&mut self, stack_frame_index : usize, label: &str) -> () {
        let children = self.get_stack_frame_by_index(stack_frame_index).children.clone();

        self.process_stack_frame(stack_frame_index, label);

        for child in children.iter() {
            self.traverse_stack_frame_children(child.clone(), label);
        }
    }

    pub fn get_stack_frame_by_index(&self, index : usize) -> &'_ StackFrame {
        return self.program_data.stack_frames.get(index).unwrap();
    }

    pub fn get_stack_frame_by_index_mut(&mut self, index : usize) -> &'_ mut StackFrame {
        return self.program_data.stack_frames.get_mut(index).unwrap();
    }
}
