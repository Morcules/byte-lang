use crate::datatypes::ast_statements::{Literal, VariableType};

// These are tokens constructed inside Tokenizer and used than in parser to create AstNodes

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub line: usize,
    pub col: usize,
    pub start_pos: usize,
    pub end_pos: usize
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keywords {
    VariableType(VariableType)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Identifiers {
    Identifier(String)
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryLocations {
    Stack,
    Register
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    EOF,
    Operator(Operators),
    Keyword(Keywords),
    Literal(Literal),
    Punctuation(Punctuations),
    BuiltInFunctions(BuiltInFunctions),
    Identifiers(Identifiers),
    MemoryLocation(MemoryLocations)
}

#[derive(Debug, PartialEq, Clone)]
pub enum BuiltInFunctions {
    Loop,
    Compare,
    Assembly,
    Format,
    StackOffset,   
    Branch,
    BranchLinked,
    
}

#[derive(Debug, PartialEq, Clone)]
pub enum Punctuations {
    Colon,
    OpenParenthesis,
    ClosedParenthesis,
    OpenBraces,
    ClosedBraces,
    OpenSquareBracket,
    ClosedSquareBracket,
    Dot,
    Comma,
    Semicolon
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operators {
    Assignment
}
