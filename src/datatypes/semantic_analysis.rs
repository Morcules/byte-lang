use std::{collections::HashMap, panic};

use crate::datatypes::{ast_statements::{AstIdentifiers, BuiltInFunctionsAst, CgBranchLinked, CgBuiltInFunctions, CgExpression, CgIdentifiers, CgStatement, CgStatementType, CgVariableAssignment, Expression, FunctionArg, Literal, StackVariableData, Statement, Statements, VariableType}, general_functions::align_memory, program_data::ProgramData, stack_frame::{StackFrame, StackVariable}, token::Identifiers};

macro_rules! throw_err {
    ($self:expr, $error:expr) => {
        $self.throw_err($error);

        return;
    };
}

pub struct SemanticAnaytis<'a> {
    program_data : &'a mut ProgramData
}

impl<'a> SemanticAnaytis<'a> {
    pub fn new(program_data : &'a mut ProgramData) -> Self {
        Self {
            program_data
        }
    }

    pub fn process_statement(&mut self, statement : &'_ Statement, stack_frame : usize) -> () {
        match statement.statement_type.clone() {
            Statements::VariableDeclaration(var_init) => {
                if let Some(init_value) = var_init.value {
                    self.initialize_identifier(&var_init.name, &init_value, stack_frame);
                };

                return;
            },
            Statements::Assignment(assignment) => {
                self.initialize_identifier(&assignment.identifier, &assignment.expression, stack_frame);
            },
            Statements::Expression(Expression::BuiltInFunction(func)) => {
                match func {
                    BuiltInFunctionsAst::BranchLinked(branch_linked) => {
                        if self.program_data.functions.get(&branch_linked.function_name).is_none() {
                            panic!("Branching to unknown function: {:?}", branch_linked.function_name);
                        }

                        let bl_function = self.program_data.functions.get(&branch_linked.function_name).unwrap().clone();

                        if branch_linked.args.len() != bl_function.args.len() {
                            throw_err!(self, "Invalid arg length");
                        }

                        let mut cg_args : Vec<CgExpression> = Vec::new();

                        let mut i = 0;
                        while i < branch_linked.args.len() {
                            let cg_expression = self.expression_to_cg(stack_frame, branch_linked.args.get(i).unwrap().clone());

                            if let Some(cg_expression_unwrapped) = cg_expression {
                                let valid : bool = match (cg_expression_unwrapped.clone(), bl_function.args.get(i).unwrap().arg_var_type.clone()) {
                                    (
                                        CgExpression::Literal(Literal::Number(_)),
                                        VariableType::I8 | VariableType::I16 | VariableType::I32
                                    ) => true,
                                    (
                                        CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data)),
                                        _
                                    ) => {
                                        stack_var_data.variable_type == bl_function.args.get(i).unwrap().arg_var_type
                                    },
                                    _ => false
                                };

                                if !valid {
                                    throw_err!(self, "Invalid args in bl");
                                }

                                cg_args.push(cg_expression_unwrapped);
                            }

                            i += 1;
                        }

                        self.add_cg_statement_to_stack_frame(stack_frame, CgStatement { statement_type: CgStatementType::BuiltInFunction(CgBuiltInFunctions::BranchLinked(CgBranchLinked{function_name: branch_linked.function_name, args: cg_args}))});
                    },
                    BuiltInFunctionsAst::Assembly(asm_expression) => {
                        let asm_code : String = match *asm_expression {
                            Expression::Literal(Literal::String(asm_code)) => asm_code,
                            Expression::BuiltInFunction(func) => {
                                func.parse(self.program_data, stack_frame).unwrap().to_string()
                            },
                            _ => {
                                throw_err!(self, "Invalid arg given to asm func");
                            }
                        };

                        self.add_cg_statement_to_stack_frame(stack_frame, CgStatement{ statement_type: CgStatementType::BuiltInFunction(CgBuiltInFunctions::Assembly(asm_code))});
                    }
                    _ => {}
                }
            }
            _ => ()
        };

        return;
    }

    pub fn expression_to_cg(&mut self, stack_frame : usize, expression : Expression) -> Option<CgExpression> {
        match expression {
            Expression::Literal(literal) => return Some(CgExpression::Literal(literal)),
            Expression::Identifier(Identifiers::Identifier(identifier)) => {
                if let Some(stack_var_ref) = self.program_data.get_stack_variable_ref(stack_frame, &identifier, 0) {
                    return Some(CgExpression::Identifier(CgIdentifiers::StackVariableData(StackVariableData{offset: stack_var_ref.local_offset, variable_type: stack_var_ref.var.variable_type})));
                } else if let Some(function_arg_ref) = self.program_data.get_function_stack_arg_ref(stack_frame, &identifier) {
                    return Some(CgExpression::Identifier(CgIdentifiers::StackVariableData(StackVariableData{offset: function_arg_ref.local_offset, variable_type: function_arg_ref.var.arg_var_type})));
                }

                self.throw_err("Variable not found");

                return None;
            },
            _ => {
                self.throw_err("Invalid Expression");

                return None;
            }
        }
    }

    pub fn process_stack_frame(&mut self, stack_frame : usize) -> () {
        let stack_mem = self.get_stack_frame_by_index_mut(stack_frame).stack_mem_allocated.clone();

        self.get_stack_frame_by_index_mut(stack_frame).stack_mem_allocated = align_memory(stack_mem, 16);

        for statement in self.get_stack_frame_by_index(stack_frame).statements.clone().iter() {
            self.process_statement(statement, stack_frame);
        }

        return;
    }

    pub fn process_stack_frame_and_children(&mut self, stack_frame_index : usize) -> () {
        self.traverse_stack_frame_children(stack_frame_index);
    }

    pub fn process_all_functions(&mut self) -> () {
        for (function_name, function) in self.program_data.functions.clone().iter() {
            self.process_stack_frame_and_children(function.first_stack_frame.clone());
        }
    }

    pub fn get_stack_variable(&self, stack_frame : usize, variable_name : &str) -> Option<StackVariable> {
        let stack_frame_borrow = self.get_stack_frame_by_index(stack_frame);

        if let Some(var) = stack_frame_borrow.variables.get(variable_name) {
            return Some(var.clone());
        } else {
            if stack_frame_borrow.parent == usize::MAX {
                return None;
            } else {
                return self.get_stack_variable(stack_frame_borrow.parent, variable_name);
            }
        }
    }

    pub fn add_cg_statement_to_stack_frame(&mut self, stack_frame : usize, statement : CgStatement) -> () {
        self.get_stack_frame_by_index_mut(stack_frame).cg_statements.push(statement);
    }

    pub fn traverse_stack_frame_children(&mut self, stack_frame_index : usize) -> () {
        let children = self.get_stack_frame_by_index(stack_frame_index).children.clone();

        for child in children.iter() {
            self.traverse_stack_frame_children(child.clone());
        }

        self.process_stack_frame(stack_frame_index);
    }

    pub fn match_var_type_with_expr(&self, variable_type : &VariableType, expr : &Expression, stack_frame : usize) -> bool {
        let res = match (variable_type.clone(), expr.clone()) {
            (VariableType::I8  | VariableType::I16 | VariableType::I32 | VariableType::I64 | VariableType::U8 | VariableType::U16 | VariableType::U32 | VariableType::U64,
            Expression::Literal(Literal::Number(_))) => true,
            (_, Expression::Identifier(Identifiers::Identifier(identifier))) => {
                if let Some(var) = self.get_stack_variable(stack_frame, &identifier) {
                    var.variable_type == variable_type.clone()
                } else if let Some(func_arg) = self.program_data.get_function_stack_arg_ref(stack_frame, &identifier) {
                    func_arg.var.arg_var_type == variable_type.clone()
                } else {
                    false
                }
            }
            _ => false
        };

        return res;
    }

    pub fn initialize_identifier(&mut self, identifier : &str, expr : &Expression, stack_frame : usize) -> () {
        if let Some(stack_var_ref) = self.program_data.get_stack_variable_ref(stack_frame, identifier, 0) {
            let init_valid = self.match_var_type_with_expr(&stack_var_ref.var.variable_type, expr, stack_frame);

            if !init_valid {
                throw_err!(self, "Invalid var declaration");
            }

            let cg_val = self.expression_to_cg(stack_frame, expr.clone());

            if let Some(cg_val_unwrapped) = cg_val {
                self.add_cg_statement_to_stack_frame(stack_frame, CgStatement{statement_type: CgStatementType::VariableAssignment(CgVariableAssignment{assign_value: cg_val_unwrapped, stack_offset: stack_var_ref.local_offset, variable_type: stack_var_ref.var.variable_type})});
            } else {
                return;
            }
        } else {
            throw_err!(self, "Expected assignment of valid identifier");
        }
    }

    pub fn throw_err(&mut self, err : &str) -> () {
        self.program_data.errors.push(String::from(err));
    }

    pub fn borrow_stack_variable_with_sf_index(&self, stack_frame : usize, variable_name : String) -> Option<&'_ StackVariable> {
        return self.get_stack_frame_by_index(stack_frame).variables.get(&variable_name);
    }

    pub fn get_stack_frame_by_index(&self, index : usize) -> &'_ StackFrame {
        return self.program_data.stack_frames.get(index).unwrap();
    }
    
    pub fn get_stack_frame_by_index_mut(&mut self, index : usize) -> &'_ mut StackFrame {
        return self.program_data.stack_frames.get_mut(index).unwrap();
    }
}
