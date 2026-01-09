use std::fmt::format;

use crate::datatypes::{assembly_instructions::asm::*, ast_statements::{AstIdentifiers, CgBuiltInFunctions, CgExpression, CgIdentifiers, CgStatement, CgStatementType, Literal, MemoryLocationsAst, VariableType}, general_functions::align_memory, program_data::{ProgramData, StackVariableRef}, stack_frame::StackFrame};

pub struct CodeGenerator<'a> {
    program_data: &'a mut ProgramData,
    stack_ptr: usize
}

impl<'a> CodeGenerator<'a> {
    pub fn new(program_data : &'a mut ProgramData) -> Self {
        return Self{program_data, stack_ptr: 0};
    }

    pub fn generate_statement(&mut self, statement : &CgStatement, stack_frame : usize) -> String {
        match statement.statement_type.clone() {
            CgStatementType::VariableAssignment(var_init) => {
                return self.init_var(var_init.stack_offset, var_init.variable_type.clone(), var_init.assign_value);
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

                        return result;
                    },
                    CgBuiltInFunctions::Assembly(assembly_code) => {
                        return assembly_code;
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

    pub fn initialize_stack_frame(&mut self, stack_frame : usize) -> String {
        let mem = self.get_stack_frame_by_index(stack_frame).stack_mem_allocated.clone();

        return create_stack_frame(mem);
    }

    pub fn return_stack_frame(&mut self, stack_frame : usize) -> String {
        let stack_frame_borrow = self.get_stack_frame_by_index(stack_frame);

        return destroy_stack_frame(stack_frame_borrow.stack_mem_allocated);
    }

    pub fn process_stack_frame(&mut self, stack_frame : usize) -> String {
        let mut result = String::new();

        result.push_str(&self.initialize_stack_frame(stack_frame));

        for statement in self.get_stack_frame_by_index(stack_frame).cg_statements.clone().iter() {
            let asm_code = self.generate_statement(statement, stack_frame);

            result.push_str(&asm_code);
        }

        result.push_str(&self.return_stack_frame(stack_frame));

        return result;
    }

    pub fn process_stack_frame_and_children(&mut self, stack_frame_index : usize) -> String {
        let compiled_code = self.traverse_stack_frame_children(stack_frame_index);

        return compiled_code;
    }

    pub fn process_all_functions(&mut self) -> String {
        let mut result = String::new();

        for (function_name, stack_frame) in self.program_data.functions.clone() {
            let function_start = format!("_{}:\n", function_name);
            result.push_str(&function_start);

            result.push_str(&self.process_stack_frame_and_children(stack_frame.first_stack_frame.clone()));
        }

        return result;
    }

    pub fn traverse_stack_frame_children(&mut self, stack_frame_index : usize) -> String {
        let mut result = String::new();

        let children = self.get_stack_frame_by_index(stack_frame_index).children.clone();

        let compiled_code = self.process_stack_frame(stack_frame_index);

        result.push_str(&compiled_code);

        for child in children.iter() {
            let child_compiled = self.traverse_stack_frame_children(child.clone());

            result.push_str(&child_compiled);
        }

        return compiled_code;
    }

    pub fn get_stack_frame_by_index(&self, index : usize) -> &'_ StackFrame {
        return self.program_data.stack_frames.get(index).unwrap();
    }

    pub fn get_stack_frame_by_index_mut(&mut self, index : usize) -> &'_ mut StackFrame {
        return self.program_data.stack_frames.get_mut(index).unwrap();
    }
}
