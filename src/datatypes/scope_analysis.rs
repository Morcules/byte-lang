use crate::datatypes::{ast_statements::{BuiltInFunctionsAst, Expression, Function, Statement, Statements, VariableDeclaration}, errors::ErrorKind, program_data::ProgramData, scope::{Scope, StackVariable}};

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

pub struct ScopeAnalysis<'a> {
    program_data : &'a mut ProgramData,
    position : usize,
    scope_stack : Vec<usize>
}

impl<'a> ScopeAnalysis<'a> {
    pub fn new(program_data : &'a mut ProgramData) -> Self {
        return Self{position : 0, scope_stack: Vec::new(), program_data};
    }

    pub fn process_statement(&mut self, statement : &mut Statement) -> () {
        if let Statements::FunctionDeclaration(func_declaration) = &mut statement.statement_type {
            if self.scope_stack.len() != 0 {
                let top_function_name = self.get_current_scope().function.clone();
                error_and_skip!(self, ErrorKind::NestedFunction, func_declaration.name.clone(), top_function_name);
            }

            if self.program_data.functions.get(&func_declaration.name).is_some() {
                error_and_skip!(self, ErrorKind::DuplicateDefinition, func_declaration.name.clone());
            }

            let scope_index = self.program_data.scopes.len();

            self.program_data.scopes.push(Scope::default(func_declaration.name.clone()));

            self.program_data.functions.insert(func_declaration.name.clone(), Function{stack_mem_allocated: 0, first_scope: scope_index, args: func_declaration.args.clone(), return_type: func_declaration.return_type.clone(), arg_stack_mem_allocated: func_declaration.args_stack_mem_allocated});

            self.scope_stack.push(scope_index);

            for func_statement in &mut func_declaration.body {
                self.process_statement(func_statement);
            }

            self.pop_scope();

            return;
        }

        match &mut statement.statement_type {
            Statements::Compare(cmp) => {
                for condition in &mut cmp.conditions {
                    let scope_index = self.create_new_scope();

                    condition.scope = scope_index;

                    for condition_statement in &mut condition.body {
                        self.process_statement(condition_statement);
                    }

                    self.pop_scope();
                }

                self.add_statement_to_current_scope(statement);
            }
            Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::BranchLinked(_))) => {
                self.add_statement_to_current_scope(statement);
            },
            Statements::VariableDeclaration(var_declaration) => {
                self.add_var_to_scope(&var_declaration);

                self.add_statement_to_current_scope(statement);
            },
            _ => {
                self.add_statement_to_current_scope(statement);
            }
        };
    }

    pub fn process_all(&mut self) -> () {
        while self.position < self.program_data.statements.len() {
            let mut current_statement = self.current_statement().clone();

            print!(" {:?} ", current_statement);
            
            self.process_statement(&mut current_statement);
            
            self.advance_position();
        }

        print!("\n");
    }

    pub fn create_new_scope(&mut self) -> usize {
        let new_scope_index = self.program_data.scopes.len();

        let parent = self.scope_stack.last().unwrap().clone();
        let function_name = self.program_data.get_scope_by_index(parent.clone()).function.clone();

        self.program_data.scopes.push(Scope::new(parent, function_name));

        self.get_current_scope().children.push(new_scope_index);

        self.scope_stack.push(new_scope_index);

        return new_scope_index;
    }

    pub fn handle_error(&mut self, err : &str) -> () {
        self.program_data.errors.push(String::from(err));

        self.advance_position();

        return;
    }

    pub fn add_statement_to_current_scope(&mut self, statement : &Statement) -> () {
        self.get_current_scope().statements.push(statement.clone());

        return;
    }

    pub fn check_if_var_exists(&mut self, var : &VariableDeclaration) -> bool {
        let mut current_scope_index : usize = self.get_current_scope_index();

        while current_scope_index != usize::MAX {
            let scope_borrow = self.program_data.get_scope_by_index(current_scope_index);

            if let Some(_) = scope_borrow.variables.get(&var.name) {
                return true;
            }

            current_scope_index = scope_borrow.parent;
        }

        return false;
    }

    pub fn add_var_to_scope(&mut self, var : &VariableDeclaration) -> () {
        if self.check_if_var_exists(var) == true {
            error_and_skip!(self, ErrorKind::DuplicateDefinition, var.name.clone());
        }

        let current_scope = self.get_current_scope();

        current_scope.variables.insert(var.name.clone(), StackVariable{variable_type: var.variable_type.clone(), variable_size: var.variable_type.get_variable_size(), offset: current_scope.stack_mem_allocated.clone()});

        current_scope.stack_mem_allocated += var.variable_type.get_variable_size();

        return;
    }

    pub fn pop_scope(&mut self) -> () {
        self.scope_stack.pop();

        return;
    }

    pub fn get_current_scope(&mut self) -> &'_ mut Scope {
        let current_index = self.get_current_scope_index();

        return self.program_data.scopes.get_mut(current_index).unwrap();
    }
    
    pub fn get_current_scope_index(&self) -> usize {
        return self.scope_stack.last().unwrap().clone();
    }

    pub fn current_statement(&mut self) -> &mut Statement {
        return self.program_data.statements.get_mut(self.position).unwrap();
    }

    pub fn advance_position(&mut self) -> () {
        self.position += 1;

        return;
    }
}
