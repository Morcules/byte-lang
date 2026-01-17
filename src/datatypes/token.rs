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
    VariableType(VariableType),

    // Is used only for erroring (for type_string func)
    VariableTypeEmpty
}

impl Keywords {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Keywords::VariableType(_) => "VariableType keyword",
            Keywords::VariableTypeEmpty => "VariableType keyword"
        };

        return String::from(str);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Identifiers {
    Identifier(String),

    // Is used only for erroring (for type_string func)
    IdentifierEmpty
}

impl Identifiers {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Identifiers::Identifier(_) => "Identifier",
            Identifiers::IdentifierEmpty => "Identifier"
        };

        return String::from(str);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryLocations {
    Stack,
    Register
}

impl MemoryLocations {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            MemoryLocations::Stack => "Stack memory location",
            MemoryLocations::Register => "Register memory location",
        };

        return String::from(str);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    EOF,
    Operator(Operators),
    Keyword(Keywords),
    Literal(Literal),
    Punctuation(Punctuations),
    BuiltInFunction(BuiltInFunctions),
    Identifier(Identifiers),
    MemoryLocation(MemoryLocations),
    OperatorEmpty,
    KeywordEmpty,
    LiteralEmpty,
    PunctuationEmpty,
    BuiltInFunctionEmpty,
    IdentifierEmpty,
    MemoryLocationEmpty
}

impl TokenType {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            TokenType::EOF => "EOF",
            TokenType::Literal(literal) => &literal.type_string(),
            TokenType::Keyword(keyword) => &keyword.type_string(),
            TokenType::Operator(operator) => &operator.type_string(),
            TokenType::Punctuation(punctuation) => &punctuation.type_string(),
            TokenType::BuiltInFunction(func) => &func.type_string(),
            TokenType::Identifier(identifier) => &identifier.type_string(),
            TokenType::MemoryLocation(memory_location) => &memory_location.type_string(),
            TokenType::LiteralEmpty => "Literal",
            TokenType::KeywordEmpty => "Keyword",
            TokenType::OperatorEmpty => "Operator",
            TokenType::PunctuationEmpty => "Punctuation",
            TokenType::BuiltInFunctionEmpty => "Built in function",
            TokenType::IdentifierEmpty => "Identifier",
            TokenType::MemoryLocationEmpty => "Memory Location"

        };

        return String::from(str);
    }
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

impl BuiltInFunctions {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            BuiltInFunctions::Loop => "Loop Built in function",
            BuiltInFunctions::Compare => "Compare Built in function",
            BuiltInFunctions::Assembly => "Assembly Built in function",
            BuiltInFunctions::Format => "Format Built in function",
            BuiltInFunctions::StackOffset => "StackOffset Built in function",   
            BuiltInFunctions::Branch => "Branch Built in function",
            BuiltInFunctions::BranchLinked => "BranchLinked Built in function",
        };

        return String::from(str);
    }
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

impl Punctuations {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Punctuations::Colon => "Colon punctuation",
            Punctuations::OpenParenthesis => "OpenParenthesis punctuation",
            Punctuations::ClosedParenthesis => "ClosedParenthesis punctuation",
            Punctuations::OpenBraces => "OpenBraces punctuation",
            Punctuations::ClosedBraces => "ClosedBraces punctuation",
            Punctuations::OpenSquareBracket => "OpenSquareBracket punctuation",
            Punctuations::ClosedSquareBracket => "ClosedSquareBracket punctuation",
            Punctuations::Dot => "Dot punctuation",
            Punctuations::Comma => "Comma punctuation",
            Punctuations::Semicolon => "Semicolon punctuation"
        };

        return String::from(str);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operators {
    Assignment
}

impl Operators {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Operators::Assignment => "Assignment operator",
        };

        return String::from(str);
    }
}
