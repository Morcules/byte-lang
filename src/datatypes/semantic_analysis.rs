use std::clone;

use crate::{datatypes::{ast_statements::{ArrayVariableData, BuiltInFunctionsAst, CgBranch, CgBranchLinked, CgBuiltInFunctions, CgCompare, CgExpression, CgIdentifiers, CgStatement, CgStatementType, CgVariableAssignment, CgVariableAssignmentType, Expression, Literal, StackVariableData, Statement, Statements, VariableType}, errors::ErrorKind, program_data::ProgramData, scope::{Scope, StackVariable}, token::Identifiers}, num_types};

macro_rules! error_and_skip {
    ($self:expr, $error:expr, $($arg:expr),+ $(,)?) => {
        $self.handle_error(&$error.format_message(&[$($arg),+]));
        return;
    };

    ($self:expr, $error:expr) => {
        $self.handle_error($error.template());

        return;
    };

}

macro_rules! error_and_none {
    ($self:expr, $error:expr, $($arg:expr),+ $(,)?) => {
        $self.handle_error(&$error.format_message(&[$($arg),+]));
        return None;
    };

    ($self:expr, $error:expr) => {
        $self.handle_error($error.template());

        return None;
    };

}

pub struct SemanticAnaytis<'a> {
    program_data : &'a mut ProgramData,
    random_label_num : usize
}

impl<'a> SemanticAnaytis<'a> {
    pub fn new(program_data : &'a mut ProgramData) -> Self {
        Self {
            program_data,
            random_label_num: 0
        }
    }

    pub fn process_statement(&mut self, statement : &'_ Statement, scope : usize) -> () {
        match statement.statement_type.clone() {
            Statements::VariableDeclaration(var_init) => {
                if let Some(init_value) = var_init.value {
                    self.initialize_identifier(Expression::Identifier(Identifiers::Identifier(var_init.name)), init_value, scope);
                };

                return;
            },
            Statements::Compare(cmp) => {
                let mut cg_expressions : [CgExpression; 2] = unsafe {
                    std::mem::zeroed()
                };

                for i in 0..2 {
                    let expr = cmp.expressions[i].clone();

                    let cg_expr = self.expression_to_cg(scope, expr);

                    if let Some(cg_expr_unwrapped) = cg_expr {
                        cg_expressions[i] = cg_expr_unwrapped;
                        continue;
                    } else {
                        error_and_skip!(self, ErrorKind::Unknown);
                    }
                }

                let scope_borrow = self.program_data.get_scope_by_index(scope);
                let func_name = if scope_borrow.parent == usize::MAX {scope_borrow.function.clone()} else {scope.to_string()};
                let label_num = self.get_random_label_num();
                let branch_name = format!("{}_{}", func_name, label_num);

                for condition in &cmp.conditions {
                    self.add_cg_statement_to_scope(condition.scope, CgStatement{statement_type: CgStatementType::BuiltInFunction(CgBuiltInFunctions::Branch(CgBranch{branch_name: branch_name.clone()}))});
                }

                self.add_cg_statement_to_scope(scope, CgStatement{statement_type: CgStatementType::Compare(CgCompare{conditions: cmp.conditions, expressions: cg_expressions, new_exit_label: branch_name})});
            },
            Statements::Assignment(assignment) => {
                self.initialize_identifier(assignment.identifier, assignment.value, scope);
            },
            Statements::Expression(Expression::BuiltInFunction(func)) => {
                match func {
                    BuiltInFunctionsAst::BranchLinked(branch_linked) => {
                        if self.program_data.functions.get(&branch_linked.function_name).is_none() {
                            error_and_skip!(self, ErrorKind::UnknownIdentifier, branch_linked.function_name);
                        }

                        let bl_function = self.program_data.functions.get(&branch_linked.function_name).unwrap().clone();

                        if branch_linked.args.len() != bl_function.args.len() {
                            error_and_skip!(self, ErrorKind::ArgumentCountMismatch, bl_function.args.len().to_string(), branch_linked.args.len().to_string(), branch_linked.function_name);
                        }

                        let mut cg_args : Vec<CgExpression> = Vec::new();

                        let mut i = 0;
                        while i < branch_linked.args.len() {
                            let cg_expression = self.expression_to_cg(scope, branch_linked.args.get(i).unwrap().clone());

                            if let Some(mut cg_expression_unwrapped) = cg_expression {
                                let var_type_expecting = bl_function.args.get(i).unwrap().arg_var_type.clone();

                                let valid = self.validate_cg_expr_with_var(&mut cg_expression_unwrapped, &var_type_expecting, scope);

                                if !valid {
                                    error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, var_type_expecting);
                                }

                                cg_args.push(cg_expression_unwrapped);
                            }

                            i += 1;
                        }

                        self.add_cg_statement_to_scope(scope, CgStatement { statement_type: CgStatementType::BuiltInFunction(CgBuiltInFunctions::BranchLinked(CgBranchLinked{function_name: branch_linked.function_name, args: cg_args}))});
                    },
                    BuiltInFunctionsAst::Assembly(asm_expression) => {
                        let asm_code : String = match *asm_expression {
                            Expression::Literal(Literal::String(asm_code)) => asm_code,
                            Expression::BuiltInFunction(func) => {
                                func.parse(self.program_data, scope).unwrap().to_string()
                            },
                            _ => {
                                error_and_skip!(self, ErrorKind::Unknown);
                            }
                        };

                        self.add_cg_statement_to_scope(scope, CgStatement{ statement_type: CgStatementType::BuiltInFunction(CgBuiltInFunctions::Assembly(asm_code))});
                    }
                    _ => {}
                }
            }
            _ => ()
        };

        return;
    }

    pub fn expression_to_cg(&mut self, scope : usize, expression : Expression) -> Option<CgExpression> {
        match expression {
            Expression::Literal(literal) => return Some(CgExpression::Literal(literal)),
            Expression::Identifier(Identifiers::Identifier(identifier)) => {
                if let Some(stack_var_ref) = self.program_data.get_stack_variable_ref(scope, &identifier) {
                    return Some(CgExpression::Identifier(CgIdentifiers::StackVariableData(StackVariableData{offset: stack_var_ref.local_offset, variable_type: stack_var_ref.var.variable_type})));
                } else if let Some(function_arg_ref) = self.program_data.get_function_stack_arg_ref(scope, &identifier) {
                    return Some(CgExpression::Identifier(CgIdentifiers::StackVariableData(StackVariableData{offset: function_arg_ref.local_offset, variable_type: function_arg_ref.var.arg_var_type})));
                }

                error_and_none!(self, ErrorKind::UnknownIdentifier, identifier);
            },
            Expression::ArrayIndex(arr_index) => {
                if let Some(stack_var_ref) = self.program_data.get_stack_variable_ref(scope, &arr_index.identifier) {
                    let VariableType::Array(_) = stack_var_ref.var.variable_type.clone() else {
                        error_and_none!(self, ErrorKind::VariableTypeMismatch, VariableType::ArrayEmpty.type_string(), stack_var_ref.var.variable_type.type_string());
                    };

                    let Some(arr_index_expr) = self.expression_to_cg(scope, *arr_index.index) else {
                        error_and_none!(self, ErrorKind::Unknown);
                    };

                    return Some(CgExpression::Identifier(CgIdentifiers::ArrayVariableData(ArrayVariableData{arr_index: Box::new(arr_index_expr), variable_type: stack_var_ref.var.variable_type, offset: stack_var_ref.local_offset})));
                } else if let Some(function_arg_ref) = self.program_data.get_function_stack_arg_ref(scope, &arr_index.identifier) {
                    let VariableType::Array(_) = function_arg_ref.var.arg_var_type.clone() else {
                        error_and_none!(self, ErrorKind::VariableTypeMismatch, VariableType::ArrayEmpty.type_string(), function_arg_ref.var.arg_var_type.type_string());
                    };

                    let Some(arr_index_expr) = self.expression_to_cg(scope, *arr_index.index) else {
                        error_and_none!(self, ErrorKind::Unknown);
                    };

                    return Some(CgExpression::Identifier(CgIdentifiers::ArrayVariableData(ArrayVariableData{arr_index: Box::new(arr_index_expr), variable_type: function_arg_ref.var.arg_var_type, offset: function_arg_ref.local_offset})));
                }

                error_and_none!(self, ErrorKind::UnknownIdentifier, arr_index.identifier);
            },
            _ => {
                error_and_none!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
            }
        }
    }

    pub fn process_scope(&mut self, scope : usize) -> () {
        for statement in self.program_data.get_scope_by_index(scope).statements.clone().iter() {
            self.process_statement(statement, scope);
        }

        return;
    }

    pub fn process_scope_and_children(&mut self, scope_index : usize) -> () {
        self.traverse_scope_children(scope_index);
    }

    pub fn process_all_functions(&mut self) -> () {
        for (function_name, function) in self.program_data.functions.clone().iter() {
            self.process_scope_and_children(function.first_scope.clone());
        }
    }

    pub fn get_stack_variable(&self, scope : usize, variable_name : &str) -> Option<StackVariable> {
        let scope_borrow = self.program_data.get_scope_by_index(scope);

        if let Some(var) = scope_borrow.variables.get(variable_name) {
            return Some(var.clone());
        } else {
            if scope_borrow.parent == usize::MAX {
                return None;
            } else {
                return self.get_stack_variable(scope_borrow.parent, variable_name);
            }
        }
    }

    pub fn add_cg_statement_to_scope(&mut self, scope : usize, statement : CgStatement) -> () {
        self.program_data.get_scope_by_index_mut(scope).cg_statements.push(statement);
    }

    pub fn traverse_scope_children(&mut self, scope_index : usize) -> () {
        let children = self.program_data.get_scope_by_index(scope_index).children.clone();

        for child in children.iter() {
            self.traverse_scope_children(child.clone());
        }

        self.process_scope(scope_index);
    }

    pub fn match_var_type_with_expr(&mut self, variable_type : &VariableType, expr : &mut Expression, scope : usize) -> bool {
        match (variable_type, expr) {
            (
                num_types!(),
                Expression::Literal(Literal::Number(_))
            ) => {
                return true;
            },
            (
                VariableType::Array(arr_type),
                Expression::Literal(Literal::Array(arr_literal))
            ) => {
                let type_expecting = *arr_type.variable_type.clone();

                for init_item in &mut arr_literal.items {
                    let valid = self.match_var_type_with_expr(&type_expecting, init_item, scope);

                    if !valid {
                        return false;
                    }

                    let cg_expr = self.expression_to_cg(scope, init_item.clone());

                    let Some(cg_expr_unwrapped) = cg_expr else {
                        return false;
                    };

                    arr_literal.cg_items.push(cg_expr_unwrapped);
                }

                return true;
            },
            (_, Expression::Identifier(Identifiers::Identifier(identifier))) => {
                if let Some(var) = self.get_stack_variable(scope, &identifier) {
                    return var.variable_type == variable_type.clone();
                } else if let Some(func_arg) = self.program_data.get_function_stack_arg_ref(scope, &identifier) {
                    return func_arg.var.arg_var_type == variable_type.clone();
                } else {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }
    }

    pub fn initialize_identifier(&mut self, var_expr : Expression, mut expr : Expression, scope : usize) -> () {
        match var_expr {
            Expression::Identifier(Identifiers::Identifier(identifier)) => {
                if let Some(stack_var_ref) = self.program_data.get_stack_variable_ref(scope, &identifier) {
                    let Some(mut cg_val) = self.expression_to_cg(scope, expr.clone()) else {
                        error_and_skip!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
                    };

                    let init_valid = self.validate_cg_expr_with_var(&mut cg_val, &stack_var_ref.var.variable_type, scope);

                    if !init_valid {
                        error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, stack_var_ref.var.variable_type.type_string());
                    }

                    self.add_cg_statement_to_scope(scope, CgStatement{statement_type: CgStatementType::VariableAssignment(CgVariableAssignment{assign_value: cg_val, assign_type: CgVariableAssignmentType::CompileTimeStackOffset(stack_var_ref.local_offset), variable_type: stack_var_ref.var.variable_type})});
                } else if let Some(function_arg_ref) = self.program_data.get_function_stack_arg_ref(scope, &identifier) {
                    let Some(mut cg_val) = self.expression_to_cg(scope, expr.clone()) else {
                        error_and_skip!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
                    };

                    let init_valid = self.validate_cg_expr_with_var(&mut cg_val, &function_arg_ref.var.arg_var_type, scope);

                    if !init_valid {
                        error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, function_arg_ref.var.arg_var_type.type_string());
                    }

                    self.add_cg_statement_to_scope(scope, CgStatement{statement_type: CgStatementType::VariableAssignment(CgVariableAssignment{assign_value: cg_val, assign_type: CgVariableAssignmentType::CompileTimeStackOffset(function_arg_ref.local_offset), variable_type: function_arg_ref.var.arg_var_type})});
                } else {
                    error_and_skip!(self, ErrorKind::UnknownIdentifier, identifier);
                }
            },
            Expression::ArrayIndex(arr_index) => {
                if let Some(stack_var_ref) = self.program_data.get_stack_variable_ref(scope, &arr_index.identifier) {
                    let Some(mut cg_val) = self.expression_to_cg(scope, expr.clone()) else {
                        error_and_skip!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
                    };

                    let VariableType::Array(arr) = stack_var_ref.var.variable_type.clone() else {
                        error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, VariableType::ArrayEmpty.type_string());
                    };

                    let init_valid = self.validate_cg_expr_with_var(&mut cg_val, &arr.variable_type, scope);

                    if !init_valid {
                        error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, arr.variable_type.type_string());
                    }

                    let Some(cg_index) = self.expression_to_cg(scope, *arr_index.index) else {
                        error_and_skip!(self, ErrorKind::Unknown);
                    };

                    self.add_cg_statement_to_scope(scope, CgStatement{statement_type: CgStatementType::VariableAssignment(CgVariableAssignment{assign_value: cg_val, assign_type: CgVariableAssignmentType::ArrayItem(ArrayVariableData{ arr_index: Box::new(cg_index), variable_type: *arr.variable_type, offset: stack_var_ref.local_offset }), variable_type: stack_var_ref.var.variable_type})});
                } else if let Some(function_arg_ref) = self.program_data.get_function_stack_arg_ref(scope, &arr_index.identifier) {
                    /*
                    TODO
                     
                    let Some(mut cg_val) = self.expression_to_cg(scope, expr.clone()) else {
                        error_and_skip!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
                    };

                    let VariableType::Array(arr) = function_arg_ref.var.arg_var_type.clone() else {
                        error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, VariableType::ArrayEmpty.type_string());
                    };

                    let init_valid = self.validate_cg_expr_with_var(&mut cg_val, &arr.variable_type, scope);

                    if !init_valid {
                        error_and_skip!(self, ErrorKind::VariableTypeMismatchExpected, arr.variable_type.type_string());
                    }

                    self.add_cg_statement_to_scope(scope, CgStatement{statement_type: CgStatementType::VariableAssignment(CgVariableAssignment{assign_value: cg_val, stack_offset: function_arg_ref.local_offset, variable_type: function_arg_ref.var.arg_var_type})});
                    */
                } else {
                    error_and_skip!(self, ErrorKind::UnknownIdentifier, arr_index.identifier);
                }
            },
            _ => {
                error_and_skip!(self, ErrorKind::Unknown);
            }
        }
    }

    pub fn validate_cg_expr_with_var(&mut self, cg_expr : &mut CgExpression, var_type : &VariableType, scope : usize) -> bool {
        let valid : bool = match (cg_expr, var_type) {
            (
                CgExpression::Literal(Literal::Array(arr_literal)),
                VariableType::Array(arr)
            ) => {
                for init_expr in &arr_literal.items {
                    let Some(mut init_cg_expr) = self.expression_to_cg(scope, init_expr.clone()) else {
                        return false;
                    };

                    if self.validate_cg_expr_with_var(&mut init_cg_expr, &*arr.variable_type, scope) == false {
                        return false;
                    }

                    arr_literal.cg_items.push(init_cg_expr);
                };

                true
            },
            (
                CgExpression::Literal(Literal::Number(_)),
                VariableType::I8 | VariableType::I16 | VariableType::I32 | VariableType::I64 |
                VariableType::U8 | VariableType::U16 | VariableType::U32 | VariableType::U64
            ) => true,
            (
                CgExpression::Identifier(CgIdentifiers::StackVariableData(stack_var_data)),
                _
            ) => {
                stack_var_data.variable_type == *var_type
            },
            (
                CgExpression::Identifier(CgIdentifiers::ArrayVariableData(arr_var_data)),
                _
            ) => {
                let VariableType::Array(arr) = arr_var_data.variable_type.clone() else {
                    return false;
                };

                *arr.variable_type == *var_type
            },
            _ => false
        };

        return valid;
    }
    
    pub fn get_random_label_num(&mut self) -> usize {
        self.random_label_num += 1;

        return self.random_label_num;
    }

    pub fn handle_error(&mut self, err : &str) -> () {
        self.program_data.errors.push(String::from(err));
    }

    pub fn borrow_stack_variable_with_scope_index(&self, scope : usize, variable_name : String) -> Option<&'_ StackVariable> {
        return self.program_data.get_scope_by_index(scope).variables.get(&variable_name);
    }
}
