use std::collections::HashMap;

use crate::datatypes::{assembly_instructions::asm::*, ast_statements::{CgBuiltInFunctions, CgExpression, CgIdentifiers, CgStatement, CgStatementType, CmpConditionType, Literal, MemoryLocationsAst, VariableType}, program_data::ProgramData, scope::Scope};

pub struct CodeGenerator<'a> {
    program_data: &'a mut ProgramData,
    stack_ptr: usize,
    labels: HashMap<String, String>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(program_data : &'a mut ProgramData) -> Self {
        return Self{program_data, stack_ptr: 0, labels: HashMap::new()};
    }

    pub fn process_statement(&mut self, statement : &CgStatement, label: &str, scope : usize) -> () {
        match statement.statement_type.clone() {
            CgStatementType::Compare(cmp) => {
                match (cmp.expressions[0].clone(), cmp.expressions[1].clone()) {
                    (CgExpression::Literal(Literal::Number(first_num)), CgExpression::Literal(Literal::Number(second_num))) => {
                        let mut result : usize = usize::MAX;

                        for condition in cmp.conditions {
                            let found_match = match condition.condition_type {
                                CmpConditionType::Equal => first_num == second_num,
                                CmpConditionType::NotEqual => first_num != second_num
                            };

                            if found_match {
                                result = condition.scope;
                                break;
                            }
                        }

                        if result == usize::MAX {
                            return;
                        }

                        self.push_compiled_code_to_label(label, &jump_to_function(&result.to_string()));

                        return;
                    },
                    _ => {}
                }

                let mut expressions = cmp.expressions.clone();

                match (expressions[0].clone(), expressions[1].clone()) {
                    (CgExpression::Literal(_), CgExpression::Identifier(CgIdentifiers::StackVariableData(_))) => {
                        expressions.swap(0, 1);
                    },
                    _ => {}
                }

                let mut result = String::new();

                result.push_str(&self.expr_to_temp_reg(&expressions[0], TempRegisters::T0));

                if let CgExpression::Literal(Literal::Number(num)) = expressions[1] {
                    result.push_str(&format!("cmp {}, #{}\n", temp_reg_for_type(VariableType::I64, false, TempRegisters::T0), num));
                } else {
                    result.push_str(&self.expr_to_temp_reg(&expressions[1], TempRegisters::T1));
                    result.push_str(&format!("cmp {}, {}\n", temp_reg_for_type(VariableType::I64, false, TempRegisters::T0), temp_reg_for_type(VariableType::I64, false, TempRegisters::T1)));
                }

                for condition in cmp.conditions {
                    result.push_str(&format!("b.{} _{}\n", condition.to_string(), condition.scope.to_string()));
                }

                result.push_str(&goto(&cmp.new_exit_label));
                result.push_str(&format!("_{}:\n", cmp.new_exit_label));

                self.push_compiled_code_to_label(label, &result);
            },
            CgStatementType::VariableAssignment(var_init) => {
                let compiled_code = &self.init_var(var_init.stack_offset, var_init.variable_type.clone(), var_init.assign_value);

                self.push_compiled_code_to_label(label, compiled_code);
            },
            CgStatementType::BuiltInFunction(built_in_function) => {
                match built_in_function {
                    CgBuiltInFunctions::Branch(branch) => {
                        self.push_compiled_code_to_label(label, &goto(&branch.branch_name));  
                    },
                    CgBuiltInFunctions::BranchLinked(branch_linked) => {
                        let mut result = String::new();

                        let function_args = self.program_data.functions.get(&branch_linked.function_name).unwrap().args.clone();
                        let function_stack_args_mem_allocated = self.program_data.functions.get(&branch_linked.function_name).unwrap().arg_stack_mem_allocated;

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
                return String::from(format!("{}{}", variable_to_reg(&temp_reg_for_type(variable_type.clone(), true, TempRegisters::T0), stack_var_data.offset, variable_type.clone()), store_reg_to_stack(&temp_reg_for_type(variable_type.clone(), false, TempRegisters::T0), target_offset, variable_type)));
            },
            (
                VariableType::Array(arr),
                CgExpression::Literal(Literal::Array(arr_expr))
            ) => {
                let mut res = String::new();

                let mut i = 0;

                let item_size = arr.variable_type.get_variable_size();

                for item in arr_expr.cg_items {
                    let init_code = self.init_var((i * item_size) + target_offset, *arr.variable_type.clone(), item);

                    res.push_str(&init_code);

                    i += 1;
                }

                return res;
            },
            _ => unreachable!()
        }
    }

    // Allocate stack memory and save return ptr in stack
    pub fn initialize_scope(&mut self, label: &str, scope : usize) -> () {
        let mem = self.program_data.get_func_stack_memory(scope);

        self.push_compiled_code_to_label(label, &create_scope(mem));
    }

    pub fn return_scope(&mut self, label: &str, scope : usize) -> () {
        let mem = self.program_data.get_func_stack_memory(scope);

        self.push_compiled_code_to_label(label, &destroy_scope(mem));

        return;
    }

    pub fn expr_to_temp_reg(&mut self, expr : &CgExpression, temp_reg : TempRegisters) -> String {
        match expr {
            CgExpression::Literal(Literal::Number(num)) => {
                return mov_num_to_reg(&temp_reg_for_type(VariableType::I64, false, temp_reg), num.clone());
            },
            CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data)) => {
                return variable_to_reg(&temp_reg_for_type(stack_var_data.variable_type.clone(), true, temp_reg), stack_var_data.offset, stack_var_data.variable_type.clone());
            },
            _ => todo!()
        }
    }

    pub fn push_compiled_code_to_label(&mut self, label: &str, code: &str) -> () {
        self.labels.get_mut(label).unwrap().push_str(code);
    }

    pub fn process_scope(&mut self, scope : usize, label: &str) -> () {
        let scope_parent = self.get_scope_by_index(scope).parent.clone();

        if scope_parent == usize::MAX {
            self.initialize_scope(label, scope);
        }

        for statement in self.get_scope_by_index(scope).cg_statements.clone().iter() {
            self.process_statement(statement, label, scope);
        }

        if scope_parent == usize::MAX {
            self.return_scope(label, scope);
        }

        return;
    }

    pub fn process_scope_and_children(&mut self, scope_index : usize, func_name : &str) -> () {
        self.traverse_scope_children(scope_index, func_name);
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
        let function_names : Vec<String> = self.program_data.functions.keys().cloned().collect();

        for function_name in function_names {
            self.labels.insert(function_name.clone(), String::new());

            let first_scope = self.program_data.functions.get(&function_name).unwrap().first_scope.clone();

            self.process_scope_and_children(first_scope, &function_name);
        }

        return;
    }

    pub fn traverse_scope_children(&mut self, scope_index : usize, label: &str) -> () {
        let children = self.get_scope_by_index(scope_index).children.clone();

        self.process_scope(scope_index, label);

        for child in children.iter() {
            self.labels.insert(child.to_string(), String::new());

            self.traverse_scope_children(child.clone(), &child.to_string());
        }
    }

    pub fn get_scope_by_index(&self, index : usize) -> &'_ Scope {
        return self.program_data.scopes.get(index).unwrap();
    }

    pub fn get_scope_by_index_mut(&mut self, index : usize) -> &'_ mut Scope {
        return self.program_data.scopes.get_mut(index).unwrap();
    }
}
