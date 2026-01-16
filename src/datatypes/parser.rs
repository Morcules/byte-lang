use std::panic;

use crate::datatypes::ast_statements::{ArrayType, Assignment, BranchLinkedAst, BuiltInFunctionsAst, CmpCondition, Compare, Expression, Format, Function, FunctionArg, FunctionDeclaration, Literal, MemoryLocationsAst, Statement, Statements, VariableDeclaration, VariableType};
use crate::datatypes::general_functions::align_memory;
use crate::datatypes::program_data::ProgramData;
use crate::datatypes::token::{BuiltInFunctions, Identifiers, Keywords, MemoryLocations, Operators, Punctuations, Token, TokenType};

macro_rules! expect_token_with_err {
    ($type_expecting:expr, $self:expr) => {
        let res = $self.expect_token($type_expecting);
        if (res.is_err()) {
            return None;
        }
    };
}

macro_rules! throw_err {
    ($self:expr, $error:expr) => {
        $self.handle_error($error);

        return None;
    };
}

pub struct Parser<'a> {
    program_data: &'a mut ProgramData,
    position: usize,
    // True = pop stack frame on nearest '};
    // False = don't pop stack frame
    scope_behaviour: Vec<bool>
}

impl<'a> Parser<'a> {
    pub fn new(program_data: &'a mut ProgramData) -> Self {
        return Self{program_data, position: 0, scope_behaviour: Vec::new()};
    }

    pub fn parse_all(&mut self) -> () {
        loop {
            match self.parse_next() {
                Some(statement) => {
                    self.program_data.statements.push(statement.clone());

                    print!("{:?}", statement);

                    if statement.statement_type == Statements::EOF {
                        break;
                    }
                },
                None => {
                    continue;
                },
            };
        }
    }

    pub fn parse_format_built_in_function(&mut self, first_token : &Token) -> Option<Statement> {
        self.advance_position();

        expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

        let TokenType::Literal(Literal::String(string_literal)) = self.current_token().kind else {
            throw_err!(self, "Expected compile time string");
        };

        self.advance_position();

        let mut args : Vec<Expression> = Vec::new();

        loop {
            match self.current_token().kind {
                TokenType::Punctuation(Punctuations::Comma) => {
                    self.advance_position();
                    match self.current_token().kind {
                        TokenType::Literal(literal) => {
                            args.push(Expression::Literal(literal));
                            self.advance_position();
                        },
                        TokenType::BuiltInFunctions(func) => {
                            let next_parsed = self.parse_next();

                            if let Some(parsed_unwrapped) = next_parsed {
                                if let Statements::Expression(expression) = parsed_unwrapped.statement_type {
                                    args.push(expression);
                                } else {
                                    throw_err!(self, "Expected expression inside format");
                                }
                            } else {
                                throw_err!(self, "Failed to parse format arg");
                            }
                        },
                        _ => {
                            throw_err!(self, "Invalid arg given to format");
                        }
                    }
                },
                TokenType::Punctuation(Punctuations::ClosedParenthesis) => {
                    self.advance_position();
                    break;
                },
                _ => {
                    throw_err!(self, "");
                }
            }
        }

        return Some(Statement::new(first_token, self.current_token().end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::Format(Format{string: string_literal, args_provided: args})))));
    }

    pub fn parse_function_declaration(&mut self, first_token : &Token, func_return_type : VariableType) -> Option<Statement> {
        self.advance_position();

        let func_name_tkn = self.current_token();

        let func_name: String = match &func_name_tkn.kind {
            TokenType::Identifiers(Identifiers::Identifier(name)) => name.clone(),
            _ => String::new(),
        };

        if func_name.is_empty() {
            throw_err!(self, "Parsing func name failed");
        }

        self.advance_position();

        expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

        let mut args : Vec<FunctionArg> = Vec::new();

        let mut stack_mem_allocated = 0;

        loop {
            match self.current_token().kind.clone() {
                TokenType::Punctuation(Punctuations::ClosedParenthesis) => break,
                TokenType::Keyword(Keywords::VariableType(var_type)) => {
                    self.advance_position();

                    let TokenType::Identifiers(Identifiers::Identifier(arg_name)) = self.current_token().kind else {
                        throw_err!(self, "Unknown token in function arg");
                    };

                    self.advance_position();

                    if TokenType::Punctuation(Punctuations::Colon) != self.current_token().kind {
                        args.push(FunctionArg { arg_var_type: var_type.clone(), arg_name, memory_location: MemoryLocationsAst::Stack(stack_mem_allocated) });
                        stack_mem_allocated += var_type.get_variable_size();
                        continue;
                    }

                    self.advance_position();

                    expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenSquareBracket), self);

                    let memory_location : MemoryLocationsAst = match self.current_token().kind {
                        TokenType::MemoryLocation(MemoryLocations::Stack) => {
                            self.advance_position();
                            stack_mem_allocated += var_type.get_variable_size();
                            MemoryLocationsAst::Stack(stack_mem_allocated - var_type.get_variable_size())
                        },
                        TokenType::MemoryLocation(MemoryLocations::Register) => {
                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                            let TokenType::Identifiers(Identifiers::Identifier(arg_register)) = self.current_token().kind else {
                                throw_err!(self, "");
                            };

                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedParenthesis), self);

                            MemoryLocationsAst::Register(arg_register)
                        },
                        _ => {
                            throw_err!(self, "");
                        }
                    };
                    
                    expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedSquareBracket), self);

                    match self.current_token().kind {
                        TokenType::Punctuation(Punctuations::Comma) => {
                            self.advance_position();
                        },
                        TokenType::Punctuation(Punctuations::ClosedParenthesis) => {}
                        _ => {
                            throw_err!(self, "Expected comma or closed parenthesis");
                        }
                    }

                    args.push(FunctionArg { arg_var_type: var_type, arg_name, memory_location });
                },
                _ => {
                    throw_err!(self, "Unknown syntax err");
                }
            };
        }

        self.advance_position();

        expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenBraces), self);

        let body : Vec<Statement> = self.parse_braces_body();

        return Some(Statement::new(first_token, self.current_token().end_pos, Statements::FunctionDeclaration(FunctionDeclaration{
            args,
            name : func_name,
            return_type: func_return_type,
            args_stack_mem_allocated: align_memory(stack_mem_allocated, 16),
            body: body
        })));
    }

    pub fn parse_variable_declaration(&mut self, first_token : &Token, var_type : VariableType, var_name : &String) -> Option<Statement> {
        if var_type == VariableType::Void {
            throw_err!(self, "Can't declare variable as void");
        }

        let end_pos;

        self.advance_position();

        let value : Option<Expression> = match self.current_token().kind {
            TokenType::Operator(operator) => {
                if operator == Operators::Assignment {
                    self.advance_position();

                    let value = self.current_token();

                    let option_value : Option<Expression> = match value.kind {
                        TokenType::Literal(identifier) => {
                            match identifier {
                                Literal::String(string_val) => Some(Expression::Literal(Literal::String(string_val))),
                                Literal::Number(num_val) => Some(Expression::Literal(Literal::Number(num_val))),
                                _ => None
                            }
                        },
                        TokenType::Identifiers(Identifiers::Identifier(identifier)) => Some(Expression::Identifier(Identifiers::Identifier(identifier))),
                        _ => None
                    };

                    if option_value.is_none() {
                        throw_err!(self, "Please Provide a valid value");
                    }

                    let value = option_value.unwrap();

                    self.advance_position();

                    let semicolon_token = self.current_token();

                    expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                    end_pos = semicolon_token.end_pos;
        
                    Some(value)
                } else {
                    throw_err!(self, "Invalid identifier found");
                }
            },
           TokenType::Punctuation(Punctuations::Semicolon) => {
                end_pos = self.current_token().end_pos;

                self.advance_position();

                None
            },
            _ => {
                throw_err!(self, "Invalid identifier found");
            }
        };

        
        return Some(Statement::new(&first_token, end_pos, Statements::VariableDeclaration(VariableDeclaration{name: var_name.clone(), value, variable_type: var_type})));
    }

    pub fn parse_next(&mut self) -> Option<Statement> {
        let token = self.current_token();

        match token.kind.clone() {
            TokenType::EOF => {
                return Some(Statement::new(&token, token.end_pos, Statements::EOF));
            },
            TokenType::BuiltInFunctions(BuiltInFunctions::Format) => {
                return self.parse_format_built_in_function(&token);
            },
            TokenType::BuiltInFunctions(BuiltInFunctions::StackOffset) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let TokenType::Identifiers(Identifiers::Identifier(var_name)) = self.current_token().kind else {
                    throw_err!(self, "Expected identifier");
                };

                self.advance_position();

                if TokenType::Punctuation(Punctuations::ClosedParenthesis) != self.current_token().kind {
                    throw_err!(self, "Expected closed parenthesis");
                };

                let end_pos = self.current_token().end_pos;

                self.advance_position();

                return Some(Statement::new(&token, end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::StackOffset(var_name)))));
            },
            TokenType::BuiltInFunctions(BuiltInFunctions::Assembly) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let asm_code = match self.current_token().kind {
                    TokenType::Literal(Literal::String(str)) => {
                        self.advance_position();

                        Expression::Literal(Literal::String(str))
                    },
                    TokenType::BuiltInFunctions(BuiltInFunctions::Format) => {
                        if let Some(format) = self.parse_next() {
                            if let Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::Format(format))) = format.statement_type {
                                Expression::BuiltInFunction(BuiltInFunctionsAst::Format(format))
                            } else {
                                throw_err!(self, "Invalid token given to function asm");
                            }
                        } else {
                            throw_err!(self, "Failed to parse format");
                        }
                    }
                    _ => {
                        throw_err!(self, "Invalid token given to function asm");
                    }
                };

                expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedParenthesis), self);
                expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                return Some(Statement::new(&token, self.current_token().end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::Assembly(Box::new(asm_code))))));
            },
            TokenType::Punctuation(Punctuations::ClosedBraces) => {
                self.advance_position();

                if self.scope_behaviour.last().unwrap().clone() == false {
                    self.scope_behaviour.pop();

                    return None;
                }

                self.scope_behaviour.pop();

                return Some(Statement::new(&token, token.end_pos, Statements::StackFramePop))
            },
            TokenType::Keyword(keyword) => {
                match keyword {
                    Keywords::VariableType(var_type) => {
                        if let VariableType::Array(_) = var_type {
                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenSquareBracket), self);

                            let TokenType::Keyword(Keywords::VariableType(arr_var_type)) = self.current_token().kind else {
                                throw_err!(self, "Exepcted var type");
                            };

                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::Comma), self);

                            let TokenType::Literal(Literal::Number(item_count)) = self.current_token().kind else {
                                throw_err!(self, "");
                            };

                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedSquareBracket), self);

                            let TokenType::Identifiers(Identifiers::Identifier(var_name)) = self.current_token().kind else {
                                throw_err!(self, "");
                            };

                            return self.parse_variable_declaration(&token, VariableType::Array(ArrayType{item_count : item_count as usize, variable_type: Box::new(arr_var_type)}), &var_name)
                        }

                        self.advance_position();

                        match self.current_token().kind {
                            TokenType::Punctuation(punctuation) => {
                                match punctuation {
                                    Punctuations::Colon => {
                                        return self.parse_function_declaration(&token, var_type);
                                        // Handle func decl
                                    },
                                    _ => {
                                        throw_err!(self, "Unknown Punctuation");
                                    }
                                }
                            },
                            TokenType::Identifiers(identifier) => {
                                match identifier {
                                    Identifiers::Identifier(var_name) => {
                                        return self.parse_variable_declaration(&token, var_type, &var_name);
                                    },
                                    _ => {
                                        throw_err!(self, "Unknown Identifier");
                                    }
                                }
                            }
                            _ => {
                                throw_err!(self, "Syntax Error");
                            }
                        }
                    }
                }
            },
            TokenType::Identifiers(Identifiers::Identifier(identifier)) => {
                self.advance_position();

                match self.current_token().kind {
                    TokenType::Operator(Operators::Assignment) => {
                        self.advance_position();

                        let expr : Expression = match self.current_token().kind {
                            TokenType::Literal(literal) => {
                                self.advance_position();

                                Expression::Literal(literal)
                            },
                            TokenType::Identifiers(Identifiers::Identifier(identifier)) => {
                                self.advance_position();

                                Expression::Identifier(Identifiers::Identifier(identifier))
                            },
                            TokenType::BuiltInFunctions(_) => {
                                let Some(parsed) = self.parse_next() else {
                                    throw_err!(self, "Expected expression after assignment");
                                };

                                let Statements::Expression(expr) = parsed.statement_type else {
                                    throw_err!(self, "Built in function invalid");
                                };

                                expr
                            },
                            _ => {
                                throw_err!(self, "Invalid expression given");
                            }
                        };

                        if TokenType::Punctuation(Punctuations::Semicolon) != self.current_token().kind {
                            throw_err!(self, "Expected semicolon after assign");
                        }

                        self.advance_position();

                        return Some(Statement::new(&token, self.current_token().end_pos, Statements::Assignment(Assignment {identifier: identifier, expression: expr})));
                    },
                    _ => {
                        throw_err!(self, "Invalid token given after identifier");
                    }
                }
            },
            TokenType::BuiltInFunctions(BuiltInFunctions::Compare) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let first_expr = self.handle_expr();

                let Some(first_expr_unwrapped) = first_expr else {
                    throw_err!(self, "invalid expr in compare");
                };

                expect_token_with_err!(TokenType::Punctuation(Punctuations::Comma), self);

                let second_expr = self.handle_expr();

                let Some(second_expr_unwrapped) = second_expr else {
                    throw_err!(self, "invalid expr in compare");
                };
                
                expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedParenthesis), self);

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenBraces), self);

                let mut conditions : Vec<CmpCondition> = Vec::new();

                let end_pos : usize;

                loop {
                    match self.current_token().kind {
                        TokenType::Punctuation(Punctuations::Dot) => {
                            self.advance_position();

                            let cmp_condition = CmpCondition::from_token(&self.current_token());
                            
                            self.advance_position();

                            let Some(mut cmp_condition_unwrapped) = cmp_condition else {
                                throw_err!(self, "Invalid cmp condition given");
                            };

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenBraces), self);

                            let cmp_condition_body = self.parse_braces_body();

                            cmp_condition_unwrapped.set_body(cmp_condition_body);

                            conditions.push(cmp_condition_unwrapped);

                            continue;
                        },
                        TokenType::Punctuation(Punctuations::ClosedBraces) => {
                            end_pos = self.current_token().end_pos.clone();

                            self.advance_position();

                            break;
                        },
                        _ => {
                            throw_err!(self, "Invalid token inside cmp");
                        }
                    }
                }

                return Some(Statement::new(&token, end_pos, Statements::Compare(Compare{conditions: conditions, expressions: [first_expr_unwrapped, second_expr_unwrapped]})))
            },
            TokenType::BuiltInFunctions(BuiltInFunctions::BranchLinked) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let TokenType::Identifiers(Identifiers::Identifier(identifier)) = self.current_token().kind else {
                    throw_err!(self, "Expected function");
                };

                self.advance_position();

                let mut args : Vec<Expression> = Vec::new();

                loop {
                    match self.current_token().kind {
                        TokenType::Punctuation(Punctuations::ClosedParenthesis) => {
                            self.advance_position();
                            break;
                        },
                        TokenType::Punctuation(Punctuations::Comma) => {
                            self.advance_position();

                            match self.current_token().kind {
                                TokenType::Literal(literal) => args.push(Expression::Literal(literal)),
                                TokenType::Identifiers(identifier) => args.push(Expression::Identifier(identifier)),
                                _ => {
                                    throw_err!(self, "");
                                }
                            }

                            self.advance_position();

                            continue;
                        },
                        _ => {
                            throw_err!(self, "Syntax Error");
                        }
                    }
                }

                let end_pos = self.current_token().end_pos;

                expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                return Some(Statement::new(&token, end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::BranchLinked(BranchLinkedAst{args, function_name: identifier})))));
            }
            _ => {
                throw_err!(self, "Syntax Error");
            }
        };
    }

    pub fn handle_expr(&mut self) -> Option<Expression> {
        match self.current_token().kind.clone() {
            TokenType::Literal(literal) => {
                self.advance_position();

                return Some(Expression::Literal(literal));
            },
            TokenType::BuiltInFunctions(_) => {
                let Some(parsed_func) = self.parse_next() else {
                    return None;
                };

                let Statements::Expression(expr) = parsed_func.statement_type else {
                    return None;
                };

                self.advance_position();

                Some(expr)
            },
            TokenType::Identifiers(Identifiers::Identifier(identifier)) => {
                self.advance_position();

                Some(Expression::Identifier(Identifiers::Identifier(identifier)))
            },
            _ => return None
        }
    }

    pub fn handle_error(&mut self, error : &str) -> () {
        self.program_data.errors.push(String::from(error));
        self.skip_until_semicolon();

        return;
    }

    pub fn skip_until_semicolon(&mut self) -> () {
        while self.current_token().kind != TokenType::Punctuation(Punctuations::Semicolon) {
            self.advance_position();
        }

        self.advance_position();

        return;
    }

    pub fn advance_position(&mut self) -> () {
        self.position += 1;
    }

    pub fn parse_braces_body(&mut self) -> Vec<Statement> {
        let mut body : Vec<Statement> = Vec::new();

        loop {
            if self.current_token().kind == TokenType::Punctuation(Punctuations::ClosedBraces) {
                self.advance_position();
                break;
            }

            let parsed = self.parse_next();

            if let Some(statement) = parsed {
                body.push(statement);
            }
        }

        return body;
    }

    pub fn current_token(&mut self) -> Token {
        let tkn = self.program_data.tokens.get(self.position).unwrap().clone(); 
        
        return tkn;
    }

    pub fn expect_token(&mut self, token_type : TokenType) -> Result<(), ()> {
        if self.current_token().kind != token_type {
            let err = format!("Current token type: {:?} Expected: {:?}", self.current_token().kind.clone(), token_type);

            self.handle_error(err.as_str());

            return Err(());
        }

        self.advance_position();

        return Ok(());
    }
}
