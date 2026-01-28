use crate::datatypes::{ast_statements::{ArrayType, Literal, VariableType}, program_data::ProgramData, token::{BuiltInFunctions, Identifiers, Keywords, MemoryLocations, Operators, Punctuations, Token, TokenType}};

// Tokenzer struct
pub struct Tokenizer<'a> {
    program_data: &'a mut ProgramData,
    position: usize,
    col: usize,
    line: usize,
}

impl<'a> Tokenizer<'a> {
    // Initialize the tokenizer.
    pub fn new(program_data: &'a mut ProgramData) -> Self {
        Self {program_data, position: 0, col: 1, line: 1}
    }

    pub fn tokenize_all(&mut self) -> () {
        loop {
            let token = self.next_token();

            match token {
                Some(tkn) => {
                    print!(" {:?} ", tkn);

                    let eof = tkn.kind == TokenType::EOF;

                    self.program_data.tokens.push(tkn);

                    if eof {
                        print!("\n");
                        return;
                    }
                },
                None => {}
            };
        }
    }

    pub fn parse_char(&mut self) -> char {
        return match self.current_char() {
            '\\' => {
                self.advance(1);
                let new_char = match self.current_char() {
                    'n' => '\n',
                    '\\' => '\\',
                    _ => ' '
                };

                self.advance(1);

                new_char
            },
            _ => {
                let cur = self.current_char();

                self.advance(1);

                return cur;
            }
        }
    }
    
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.program_data.source_code.len() <= self.position {
            return Some(Token{kind: TokenType::EOF, col: self.col, line: self.line, start_pos: self.position, end_pos: self.position});
        }

        let mut res = String::new();

        let start_pos = self.position;

        match self.current_char() {
            '\n' | ';' | '(' | ')' | ',' | '[' | ']' | '{' | '}' | '.' => {
                res = String::from(self.current_char());
                self.advance(1);
            },
            '/' => {
                if self.char_at_offset(1) == '/' {
                    self.advance(2);
                    
                    loop {
                        if self.current_char() == '/' && self.char_at_offset(1) == '/' {
                            self.advance(2);

                            break;
                        }

                        self.advance(1);  
                    };

                    return None;
                }
            }
            '\'' => {
                self.advance(1);

                let char = self.parse_char();

                let literal_res = char as u8;

                if self.current_char() != '\'' {
                    panic!()
                }

                self.advance(1);

                return Some(Token{kind: TokenType::Literal(Literal::Number(literal_res as i64)), col: self.col, line: self.line, start_pos, end_pos: self.position});
            },
            '"' => {
                let mut str = String::new();

                self.advance(1);

                while self.position < self.program_data.source_code.len() && self.current_char() != '"' {
                    match self.current_char() {
                        '"' => {
                            break;
                        },
                        _ => {
                            str.push(self.parse_char());
                        }
                    }
                };

                self.advance(1);

                return Some(Token{kind: TokenType::Literal(Literal::String(str)), col: self.col, line: self.line, start_pos, end_pos: self.position});
            },
            _ => {
                while self.position < self.program_data.source_code.len() && self.current_char().is_whitespace() == false && matches!(self.current_char(), ';' | '(' | ')' | ',' | '[' | ']' | '{' | '}') == false {
                    res.push(self.current_char());
                    self.advance(1);
                };
            }
        }

        let token_default = Token{kind: TokenType::EOF, col: self.col, line: self.line, start_pos, end_pos: self.position};

        match &res as &str {
            "\n" => {
                self.line += 1;
                self.col = 1;
            },
            "stack_offset" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::StackOffset), ..token_default});
            }
            ";" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::Semicolon), ..token_default});
            },
            ":" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::Colon), ..token_default});
            }
            "=" => {
                return Some(Token{kind: TokenType::Operator(Operators::Assignment), ..token_default});
            },
            "{" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::OpenBraces), ..token_default});
            },
            "}" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::ClosedBraces), ..token_default});
            },
            "(" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::OpenParenthesis), ..token_default});
            },
            ")" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::ClosedParenthesis), ..token_default});
            },
            "[" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::OpenSquareBracket), ..token_default});
            }
            "]" => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::ClosedSquareBracket), ..token_default});
            }
            "," => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::Comma), ..token_default});
            },
            "." => {
                return Some(Token{kind: TokenType::Punctuation(Punctuations::Dot), ..token_default});
            },
            "cmp" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::Compare), ..token_default});
            },
            "bl" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::BranchLinked), ..token_default});
            },
            "stack"=> {
                return Some(Token{kind: TokenType::MemoryLocation(MemoryLocations::Stack), ..token_default});
            },
            "reg"=> {
                return Some(Token{kind: TokenType::MemoryLocation(MemoryLocations::Register), ..token_default});
            },
            "asm" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::Assembly), ..token_default});
            }
            "i64" | "i32" | "i16" | "i8" | "u64" | "u32" | "u16" | "u8" | "void" | "array" => {
                return Some(Token{kind: TokenType::Keyword(Keywords::VariableType(
                    match &res as &str {
                        "i64" => VariableType::I64,
                        "i32" => VariableType::I32,
                        "i16" => VariableType::I16,
                        "i8" => VariableType::I8,
                        "u64" => VariableType::U64,
                        "u32" => VariableType::U32,
                        "u16" => VariableType::U16,
                        "u8" => VariableType::U8,
                        "void" => VariableType::Void,
                        "array" => VariableType::Array(ArrayType::none()),
                        _ => unreachable!()
                    }
                )), ..token_default});
            },
            "format" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::Format), ..token_default});
            }
            "compare" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::Compare), ..token_default});
            },
            "loop" => {
                return Some(Token{kind: TokenType::BuiltInFunction(BuiltInFunctions::Loop), ..token_default});
            },
            _ => {
                match res.parse::<i64>() {
                    Ok(num) => {
                        return Some(Token{kind: TokenType::Literal(Literal::Number(num)), ..token_default});
                    },
                    Err(_) => {
                        return Some(Token{kind: TokenType::Identifier(Identifiers::Identifier(res)), ..token_default});

                    }
                }
            }
        } 

        return None;
    }

    // Increment position by X amount
    pub fn advance(&mut self, num : usize) {
        let mut new_position : usize = self.position;
        let mut new_col : usize = self.col;
        let mut new_line : usize = self.line;

        for i in 0..num {
            if self.char_at_offset(i as i32) == '\n' {
                new_col = 1;
                new_line += 1;
            } else {
                new_col += 1;
            }

            new_position += 1;
        }

        self.position = new_position;
        self.col = new_col;
        self.line = new_line;
    }

    // Skips whitespace.
    pub fn skip_whitespace(&mut self) {
        while self.position < self.program_data.source_code.len() && self.current_char().is_whitespace() && self.current_char() != '\n' {
            self.col += 1;
            self.position += 1;
        }
    }

    // Get current char of input.
    pub fn current_char(&self) -> char {
        self.program_data.source_code[self.position..].chars().next().unwrap()
    }

    pub fn char_at_offset(&self, offset : i32) -> char {
        self.program_data.source_code[((self.position as i32) + offset) as usize..].chars().next().unwrap()
    }
 }
