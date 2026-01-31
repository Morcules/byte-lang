use std::fmt::format;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use crate::datatypes::ast_statements::{ArrayIndex, ArrayLiteral, ArrayType, Assignment, BranchLinkedAst, BuiltInFunctionsAst, CmpCondition, Compare, Expression, Format, FunctionArg, FunctionDeclaration, Literal, MemoryLocationsAst, Statement, Statements, VariableDeclaration, VariableType};
use crate::datatypes::errors::ErrorKind;
use crate::datatypes::general_functions::align_memory;
use crate::datatypes::program_data::ProgramData;
use crate::datatypes::token::{BuiltInFunctions, Identifiers, Keywords, MemoryLocations, Operators, Punctuations, Token, TokenType};
use crate::err_args;

macro_rules! expect_token_with_err {
    ($type_expecting:expr, $self:expr) => {
        let res = $self.expect_token($type_expecting);
        if (res.is_err()) {
            return None;
        }
    };
}

macro_rules! error_and_skip {
    ($self:expr, $error:expr, $($arg:expr),+ $(,)?) => {
        $self.handle_error(&$error.format_message(&[$($arg),+]));
        return None;
    };

    ($self:expr, $error:expr) => {
        $self.handle_error($error.template());

        return None;
    };

}

pub struct Parser<'a> {
    program_data: &'a mut ProgramData,
    position: usize,
    file_name: String
}

impl<'a> Parser<'a> {
    pub fn new(program_data: &'a mut ProgramData, file_name: &str) -> Self {
        return Self{program_data, position: 0, file_name: String::from(file_name)};
    }

    pub fn parse_all(&mut self) -> () {
        while self.position < self.program_data.tokens.len() {
            match self.parse_next() {
                Some(statement) => {
                    self.program_data.statements.push(statement);
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

        let format_string_tkn = self.current_token();

        let TokenType::Literal(Literal::String(string_literal)) = format_string_tkn.kind else {
            error_and_skip!(self, ErrorKind::InvalidToken, Literal::StringEmpty.type_string(), format_string_tkn.kind.type_string());
        };

        self.advance_position();

        let mut args : Vec<Expression> = Vec::new();

        loop {
            match self.current_token().kind {
                TokenType::Punctuation(Punctuations::Comma) => {
                    self.advance_position();

                    let arg_token = self.current_token();

                    match arg_token.kind {
                        TokenType::Literal(literal) => {
                            args.push(Expression::Literal(literal));
                            self.advance_position();
                        },
                        TokenType::BuiltInFunction(_) => {
                            let next_parsed = self.parse_next();

                            if let Some(parsed_unwrapped) = next_parsed {
                                if let Statements::Expression(expression) = parsed_unwrapped.statement_type {
                                    args.push(expression);
                                } else {
                                    error_and_skip!(self, ErrorKind::InvalidToken, Statements::ExpressionEmpty.type_string(), parsed_unwrapped.statement_type.type_string());
                                }
                            } else {
                                // Don't error, because parsing failing already prints error and
                                // skips
                                return None;
                            }
                        },
                        _ => {
                            error_and_skip!(self, ErrorKind::InvalidToken, Statements::ExpressionEmpty.type_string(), arg_token.kind.type_string());
                        }
                    }
                },
                TokenType::Punctuation(Punctuations::ClosedParenthesis) => {
                    self.advance_position();
                    break;
                },
                _ => {
                    error_and_skip!(self, ErrorKind::InvalidToken, TokenType::Punctuation(Punctuations::ClosedParenthesis));
                }
            }
        }

        return Some(Statement::new(first_token, self.current_token().end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::Format(Format{string: string_literal, args_provided: args})))));
    }

    pub fn parse_function_declaration(&mut self, first_token : &Token, func_return_type : VariableType) -> Option<Statement> {
        self.advance_position();

        let func_name_tkn = self.current_token();

        let func_name: String = match &func_name_tkn.kind {
            TokenType::Identifier(Identifiers::Identifier(name)) => name.clone(),
            _ => String::new(),
        };

        if func_name.is_empty() {
            error_and_skip!(self, ErrorKind::InvalidToken, TokenType::Identifier(Identifiers::IdentifierEmpty).type_string(), func_name_tkn.kind.type_string());
        }

        self.advance_position();

        expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

        let mut args : Vec<FunctionArg> = Vec::new();

        let mut stack_mem_allocated = 0;

        loop {
            match &self.current_token().kind {
                TokenType::Punctuation(Punctuations::ClosedParenthesis) => break,
                TokenType::Keyword(Keywords::VariableType(var_type)) => {
                    self.advance_position();

                    let arg_name_tkn = self.current_token();

                    let TokenType::Identifier(Identifiers::Identifier(arg_name)) = arg_name_tkn.kind else {
                        error_and_skip!(self, ErrorKind::InvalidToken, TokenType::Identifier(Identifiers::IdentifierEmpty).type_string(), arg_name_tkn.kind.type_string());
                    };

                    self.advance_position();

                    if TokenType::Punctuation(Punctuations::Colon) != self.current_token().kind {
                        args.push(FunctionArg { arg_var_type: var_type.clone(), arg_name, memory_location: MemoryLocationsAst::Stack(stack_mem_allocated) });
                        stack_mem_allocated += var_type.get_variable_size();
                        continue;
                    }

                    self.advance_position();

                    expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenSquareBracket), self);

                    let memory_loc_tkn = self.current_token();

                    let memory_location : MemoryLocationsAst = match memory_loc_tkn.kind {
                        TokenType::MemoryLocation(MemoryLocations::Stack) => {
                            self.advance_position();
                            stack_mem_allocated += var_type.get_variable_size();
                            MemoryLocationsAst::Stack(stack_mem_allocated - var_type.get_variable_size())
                        },
                        TokenType::MemoryLocation(MemoryLocations::Register) => {
                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                            let arg_reg_tkn = self.current_token();

                            let TokenType::Identifier(Identifiers::Identifier(arg_register)) = arg_reg_tkn.kind else {
                                error_and_skip!(self, ErrorKind::InvalidToken, TokenType::Identifier(Identifiers::IdentifierEmpty).type_string(), arg_reg_tkn.kind.type_string());
                            };

                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedParenthesis), self);

                            MemoryLocationsAst::Register(arg_register)
                        },
                        _ => {
                            error_and_skip!(self, ErrorKind::InvalidToken, TokenType::MemoryLocationEmpty.type_string(), memory_loc_tkn.kind.type_string());
                        }
                    };
                    
                    expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedSquareBracket), self);

                    let next_tkn = self.current_token();

                    match next_tkn.kind {
                        TokenType::Punctuation(Punctuations::Comma) => {
                            self.advance_position();
                        },
                        TokenType::Punctuation(Punctuations::ClosedParenthesis) => {}
                        _ => {
                            error_and_skip!(self, ErrorKind::InvalidToken, TokenType::Punctuation(Punctuations::ClosedParenthesis), next_tkn.kind);
                        }
                    }

                    args.push(FunctionArg { arg_var_type: var_type.clone(), arg_name, memory_location });
                },
                _ => {
                    error_and_skip!(self, ErrorKind::Unknown);
                }
            };
        }

        self.advance_position();

        expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenBraces), self);

        let body : Vec<Statement> = self.parse_braces_body();

        return Some(Statement::new(first_token, self.position - 1, Statements::FunctionDeclaration(FunctionDeclaration{
            args,
            name : func_name,
            return_type: func_return_type,
            args_stack_mem_allocated: align_memory(stack_mem_allocated, 16),
            body: body
        })));
    }

    pub fn parse_variable_declaration(&mut self, first_token : &Token, var_type : VariableType, var_name : &String) -> Option<Statement> {
        if var_type == VariableType::Void {
            error_and_skip!(self, ErrorKind::VariableCannotBeVoid);
        }

        let end_pos;

        self.advance_position();

        let value_tkn = self.current_token();

        let value : Option<Expression> = match value_tkn.kind {
            TokenType::Operator(operator) => {
                if operator == Operators::Assignment {
                    self.advance_position();

                    let option_value = self.parse_expr();

                    if option_value.is_none() {
                        error_and_skip!(self, ErrorKind::ExpectedToken, Statements::ExpressionEmpty.type_string());
                    }

                    let value = option_value.unwrap();

                    let semicolon_token = self.current_token();

                    expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                    end_pos = semicolon_token.end_pos;
        
                    Some(value)
                } else {
                    error_and_skip!(self, ErrorKind::InvalidToken, TokenType::OperatorEmpty.type_string(), operator.type_string());
                }
            },
           TokenType::Punctuation(Punctuations::Semicolon) => {
                end_pos = self.current_token().end_pos;

                self.advance_position();

                None
            },
            _ => {
                error_and_skip!(self, ErrorKind::Unknown);
            }
        };

        
        return Some(Statement::new(&first_token, end_pos, Statements::VariableDeclaration(VariableDeclaration{name: var_name.clone(), value, variable_type: var_type})));
    }

    pub fn parse_next(&mut self) -> Option<Statement> {
        let token = self.current_token();

        match token.kind.clone() {
            TokenType::BuiltInFunction(BuiltInFunctions::Format) => {
                return self.parse_format_built_in_function(&token);
            },
            TokenType::BuiltInFunction(BuiltInFunctions::StackOffset) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let var_name_tkn = self.current_token();

                let TokenType::Identifier(Identifiers::Identifier(var_name)) = var_name_tkn.kind else {
                    error_and_skip!(self, ErrorKind::InvalidToken, TokenType::IdentifierEmpty.type_string(), var_name_tkn.kind.type_string());
                };

                self.advance_position();

                let end_pos = self.current_token().end_pos;

                expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedParenthesis), self);

                return Some(Statement::new(&token, end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::StackOffset(var_name)))));
            },
            TokenType::BuiltInFunction(BuiltInFunctions::Assembly) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let asm_code = match self.current_token().kind {
                    TokenType::Literal(Literal::String(str)) => {
                        self.advance_position();

                        Expression::Literal(Literal::String(str))
                    },
                    TokenType::BuiltInFunction(BuiltInFunctions::Format) => {
                        if let Some(format) = self.parse_next() {
                            if let Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::Format(format))) = format.statement_type {
                                Expression::BuiltInFunction(BuiltInFunctionsAst::Format(format))
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => {
                        error_and_skip!(self, ErrorKind::InvalidToken, Statements::ExpressionEmpty.type_string(), token.kind.type_string());
                    }
                };

                expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedParenthesis), self);
                expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                return Some(Statement::new(&token, self.current_token().end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::Assembly(Box::new(asm_code))))));
            },
            TokenType::Keyword(keyword) => {
                match keyword {
                    Keywords::VariableType(var_type) => {
                        if let VariableType::Array(_) = var_type {
                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenSquareBracket), self);

                            let var_type_tkn = self.current_token();

                            let TokenType::Keyword(Keywords::VariableType(arr_var_type)) = var_type_tkn.kind else {
                                error_and_skip!(self, ErrorKind::InvalidToken, Keywords::VariableTypeEmpty.type_string(), var_type_tkn.kind.type_string());
                            };

                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::Comma), self);

                            let item_count_tkn = self.current_token();

                            let TokenType::Literal(Literal::Number(item_count)) = item_count_tkn.kind else {
                                error_and_skip!(self, ErrorKind::InvalidToken, Literal::NumberEmpty.to_string(), item_count_tkn.kind.type_string());
                            };

                            self.advance_position();

                            expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedSquareBracket), self);

                            let var_name_tkn = self.current_token();

                            let TokenType::Identifier(Identifiers::Identifier(var_name)) = var_name_tkn.kind else {
                                error_and_skip!(self, ErrorKind::InvalidToken, TokenType::IdentifierEmpty.type_string(), var_name_tkn.kind.type_string());
                            };

                            return self.parse_variable_declaration(&token, VariableType::Array(ArrayType{item_count : item_count as usize, variable_type: Box::new(arr_var_type)}), &var_name)
                        }

                        self.advance_position();

                        let punc_tkn = self.current_token();

                        match punc_tkn.kind.clone() {
                            TokenType::Punctuation(punctuation) => {
                                match punctuation {
                                    Punctuations::Colon => {
                                        return self.parse_function_declaration(&token, var_type);
                                        // Handle func decl
                                    },
                                    _ => {
                                        error_and_skip!(self, ErrorKind::InvalidToken, TokenType::Punctuation(Punctuations::Colon), punc_tkn.kind);
                                    }
                                }
                            },
                            TokenType::Identifier(identifier) => {
                                match identifier {
                                    Identifiers::Identifier(var_name) => {
                                        return self.parse_variable_declaration(&token, var_type, &var_name);
                                    },
                                    _ => {
                                        error_and_skip!(self, ErrorKind::InvalidToken, TokenType::IdentifierEmpty.type_string(), identifier.type_string());
                                    }
                                }
                            }
                            _ => {
                                error_and_skip!(self, ErrorKind::Unknown);
                            }
                        }
                    },
                    _ => {
                        error_and_skip!(self, ErrorKind::Unknown);
                    }
                }
            },
            TokenType::Identifier(Identifiers::Identifier(_)) => {
                let Some(identifier_expr) = self.parse_expr() else {
                    error_and_skip!(self, ErrorKind::Unknown);
                };

                match self.current_token().kind {
                    TokenType::Operator(Operators::Assignment) => {
                        self.advance_position();

                        let Some(assign_value) = self.parse_expr() else {
                            error_and_skip!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
                        };

                        let end_pos = self.current_token().end_pos.clone();

                        expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                        return Some(Statement::new(&token, end_pos, Statements::Assignment(Assignment{identifier: identifier_expr, value: assign_value})));
                    },
                    _ => {
                        error_and_skip!(self, ErrorKind::Unknown);
                    }
                }
            },
            TokenType::BuiltInFunction(BuiltInFunctions::Compare) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let first_expr = self.handle_expr();

                let Some(first_expr_unwrapped) = first_expr else {
                    error_and_skip!(self, ErrorKind::ExpectedToken, Statements::ExpressionEmpty.type_string());
                };

                expect_token_with_err!(TokenType::Punctuation(Punctuations::Comma), self);

                let second_expr = self.handle_expr();

                let Some(second_expr_unwrapped) = second_expr else {
                    error_and_skip!(self, ErrorKind::ExpectedToken, Statements::ExpressionEmpty.type_string());
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
                                error_and_skip!(self, ErrorKind::ExpectedToken, Statements::CompareEmpty.type_string());
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
                            error_and_skip!(self, ErrorKind::Unknown);
                        }
                    }
                }

                return Some(Statement::new(&token, end_pos, Statements::Compare(Compare{conditions: conditions, expressions: [first_expr_unwrapped, second_expr_unwrapped]})))
            },
            TokenType::Punctuation(Punctuations::Hashtag) => {
                self.advance_position();

                let TokenType::Identifier(Identifiers::Identifier(directive)) = self.current_token().kind else {
                    error_and_skip!(self, ErrorKind::Unknown);
                };

                self.advance_position();

                match directive.as_str() {
                    "import" => {
                        let TokenType::Literal(Literal::String(file_location)) = self.current_token().kind else {
                            error_and_skip!(self, ErrorKind::ExpectedToken, TokenType::IdentifierEmpty.type_string());
                        };

                        self.advance_position();
                        
                        expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                        let current_file = Path::new(&self.file_name);

                        let parent_dir = current_file.parent().expect("Current file has no parent directory");

                        let abs_path = parent_dir.join(file_location);

                        let abs_path = fs::canonicalize(&abs_path).expect("File does not exist");

                        println!("Absolute path: {}", abs_path.display());

                        let abs_path = abs_path.to_str().unwrap();

                        self.program_data.source_codes.insert(String::from(abs_path), String::new());

                        let mut file = File::open(abs_path).expect("Error Oppening File");

                        file.read_to_string(self.program_data.source_codes.get_mut(abs_path).unwrap()).unwrap();

                        return None;
                    },
                    _ => {
                        error_and_skip!(self, ErrorKind::Unknown);
                    }
                };
            },
            TokenType::BuiltInFunction(BuiltInFunctions::BranchLinked) => {
                self.advance_position();

                expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenParenthesis), self);

                let identifier_tkn = self.current_token();

                let TokenType::Identifier(Identifiers::Identifier(identifier)) = identifier_tkn.kind else {
                    error_and_skip!(self, ErrorKind::InvalidToken, TokenType::IdentifierEmpty.type_string(), identifier_tkn.kind.type_string());
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

                            let expr_tkn = self.current_token();

                            match expr_tkn.kind {
                                TokenType::Literal(literal) => args.push(Expression::Literal(literal)),
                                TokenType::Identifier(identifier) => args.push(Expression::Identifier(identifier)),
                                _ => {
                                    error_and_skip!(self, ErrorKind::InvalidToken, Statements::ExpressionEmpty.type_string(), expr_tkn.kind.type_string());
                                }
                            }

                            self.advance_position();

                            continue;
                        },
                        _ => {
                            error_and_skip!(self, ErrorKind::Unknown);
                        }
                    }
                }

                let end_pos = self.current_token().end_pos;

                expect_token_with_err!(TokenType::Punctuation(Punctuations::Semicolon), self);

                return Some(Statement::new(&token, end_pos, Statements::Expression(Expression::BuiltInFunction(BuiltInFunctionsAst::BranchLinked(BranchLinkedAst{args, function_name: identifier})))));
            }
            _ => {
                error_and_skip!(self, ErrorKind::Unknown);
            }
        };
    }

    pub fn handle_expr(&mut self) -> Option<Expression> {
        match self.current_token().kind.clone() {
            TokenType::Literal(literal) => {
                self.advance_position();

                return Some(Expression::Literal(literal));
            },
            TokenType::BuiltInFunction(_) => {
                let Some(parsed_func) = self.parse_next() else {
                    return None;
                };

                let Statements::Expression(expr) = parsed_func.statement_type else {
                    return None;
                };

                self.advance_position();

                Some(expr)
            },
            TokenType::Identifier(Identifiers::Identifier(identifier)) => {
                self.advance_position();

                Some(Expression::Identifier(Identifiers::Identifier(identifier)))
            },
            _ => return None
        }
    }

    pub fn handle_error(&mut self, error : &str) -> () {
        self.program_data.errors.push(String::from(error));
        self.skip_until_semicolon();

        self.advance_position();

        return;
    }

    pub fn skip_statement(&mut self) -> () {
        self.skip_until_semicolon();

        self.advance_position();

        return;
    }

    pub fn skip_until_semicolon(&mut self) -> () {
        while self.current_token().kind != TokenType::Punctuation(Punctuations::Semicolon) {
            self.advance_position();
        }

        return;
    }

    pub fn advance_position(&mut self) -> () {
        self.position += 1;
    }

    pub fn parse_array_init_string(&mut self) -> Option<Expression> {
        let TokenType::Literal(Literal::String(string)) = self.current_token().kind else {
            error_and_skip!(self, ErrorKind::ExpectedToken, TokenType::Literal(Literal::StringEmpty).type_string());
        };

        self.advance_position();

        let mut result = ArrayLiteral{items: Vec::new(), cg_items: Vec::new()};

        for char in string.chars() {
            result.items.push(Expression::Literal(Literal::Number(char as i64)));
        }

        return Some(Expression::Literal(Literal::Array(result)));

    }

    pub fn parse_array_init(&mut self) -> Option<Expression> {
        expect_token_with_err!(TokenType::Punctuation(Punctuations::OpenSquareBracket), self);

        let mut result = ArrayLiteral{items: Vec::new(), cg_items: Vec::new()};

        loop {
            let expr = self.parse_expr();

            let Some(unwrapped_expr) = expr else {
                error_and_skip!(self, ErrorKind::ExpectedToken, Statements::ExpressionEmpty.type_string());
            };

            result.items.push(unwrapped_expr);

            match self.current_token().kind {
                TokenType::Punctuation(Punctuations::ClosedSquareBracket) => {
                    self.advance_position();

                    if result.items.len() == 0 {
                        return None;
                    }

                    break;
                },
                TokenType::Punctuation(Punctuations::Comma) => {
                    self.advance_position();
                    
                    continue;
                }
                _ => {
                    error_and_skip!(self, ErrorKind::Unknown);
                }
            }
        }

        return Some(Expression::Literal(Literal::Array(result)));
    }

    pub fn parse_expr(&mut self) -> Option<Expression> {
        let result : Option<Expression> = match self.current_token().kind {
            TokenType::Literal(literal) => {
                match literal {
                    Literal::String(string_val) => self.parse_array_init_string(),
                    Literal::Number(num_val) => {
                        self.advance_position();
                        Some(Expression::Literal(Literal::Number(num_val)))
                    },
                    _ => None
                }
            },
            TokenType::Punctuation(Punctuations::OpenSquareBracket) => {
                let arr_init = self.parse_array_init();

                arr_init
            },
            TokenType::Identifier(Identifiers::Identifier(identifier)) => {
                self.advance_position();

                match self.current_token().kind {
                    TokenType::Punctuation(Punctuations::OpenSquareBracket) => {
                        self.advance_position();

                        let Some(arr_inx) = self.parse_expr() else {
                            error_and_skip!(self, ErrorKind::ExpectedStatement, Statements::ExpressionEmpty.type_string());
                        };

                        expect_token_with_err!(TokenType::Punctuation(Punctuations::ClosedSquareBracket), self);


                        Some(Expression::ArrayIndex(ArrayIndex{identifier: identifier, index: Box::new(arr_inx)}))
                    },
                    _ => {
                        Some(Expression::Identifier(Identifiers::Identifier(identifier)))
                    }
                }
            }
            _ => None
        };

        return result;
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
            let err = ErrorKind::InvalidToken.format_message(err_args!(token_type, self.current_token().kind));

            self.handle_error(err.as_str());

            return Err(());
        }

        self.advance_position();

        return Ok(());
    }
}
