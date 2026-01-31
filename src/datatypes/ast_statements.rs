use std::panic;

use crate::datatypes::{program_data::ProgramData, token::{Identifiers, Token, TokenType}};

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub col: usize,
    pub line: usize,
    pub start_pos: usize,
    pub end_pos: usize,
    pub statement_type: Statements
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub return_type: VariableType,
    pub args: Vec<FunctionArg>,
    pub first_scope: usize,
    pub arg_stack_mem_allocated: usize,
    pub stack_mem_allocated: usize
}

impl Statement {
    #[inline]
    pub fn new(token: &Token, end_pos: usize, statement_type: Statements) -> Self {
        Self {
            col: token.col,
            line: token.line,
            start_pos: token.start_pos,
            end_pos,
            statement_type,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstIdentifiers {
    StackVariableIdentifier(String),
    FunctionArgIdentifier(String)
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionArg {
    pub arg_var_type : VariableType,
    pub arg_name : String,
    pub memory_location : MemoryLocationsAst
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub args : Vec<FunctionArg>,
    pub name : String,
    pub return_type : VariableType,
    pub body : Vec<Statement>,
    pub args_stack_mem_allocated : usize
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub identifier : Expression,
    pub value : Expression
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statements {
    Assignment(Assignment),
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    Compare(Compare),
    StackFramePop,
    Expression(Expression),

    // Is used only for erroring (for type_string func)
    AssignmentEmpty,
    VariableDeclarationEmpty,
    FunctionDeclarationEmpty,
    CompareEmpty,
    ExpressionEmpty,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BuiltInFunctionsAst {
    Assembly(Box<Expression>),
    Format(Format),
    StackOffset(String),
    BranchLinked(BranchLinkedAst),

    // Is used only for erroring (for type_string func)
    AssemblyEmpty,
    FormatEmpty,
    StackOffsetEmpty,
    BranchLinkedEmpty,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Compare {
    pub expressions : [Expression; 2],
    pub conditions : Vec<CmpCondition>
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryLocationsAst {
    Stack(usize),
    Register(String)
}

#[derive(Debug, PartialEq, Clone)]
pub struct BranchLinkedAst {
    pub args : Vec<Expression>,
    pub function_name : String
}

impl BuiltInFunctionsAst {
    pub fn parse(&self, program_data : &mut ProgramData, scope : usize) -> Option<Literal> {
        return match self {
            BuiltInFunctionsAst::StackOffset(identifier) => {
                if let Some(var) = program_data.get_stack_variable_ref(scope, identifier) {
                    Some(Literal::Number(var.local_offset as i64))
                } else if let Some(arg) = program_data.get_function_stack_arg_ref(scope, identifier) {
                    Some(Literal::Number(arg.local_offset as i64))
                } else {
                    panic!()
                }
            },
            BuiltInFunctionsAst::Format(format) => {
                let mut result = String::new();

                let mut position : usize = 0;

                for arg in format.args_provided.clone() {
                    loop {
                        match format.string.chars().nth(position).unwrap() {
                            '{' => {
                                position += 1;

                                if format.string.chars().nth(position).unwrap() != '}' {
                                    panic!("Expected '}}' after '{{' in format");
                                }

                                position += 1;

                                match arg {
                                    Expression::Literal(Literal::Number(num)) => {
                                        result.push_str(&num.to_string());
                                    },
                                    Expression::Literal(Literal::String(string)) => {
                                        result.push_str(&string);
                                    },
                                    Expression::BuiltInFunction(func) => {
                                        let parsed_func = func.parse(program_data, scope);
                                        result.push_str(&parsed_func.unwrap().to_string());
                                    }
                                    _ => {
                                        panic!("Invalid Format");
                                    }
                                }

                                break;
                            },
                            _ => {
                                result.push(format.string.chars().nth(position).unwrap());

                                position += 1;
                            }
                        }
                    }
                }

                while position < format.string.len() {
                    result.push(format.string.chars().nth(position).unwrap());
                    position += 1;
                }

                return Some(Literal::String(result));

            }
            _ => unreachable!()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Format {
    pub string : String,
    pub args_provided : Vec<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub variable_type: VariableType,
    pub value: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayIndex {
    pub identifier : String,
    pub index : Box<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    ArrayIndex(ArrayIndex),
    Literal(Literal),
    Identifier(Identifiers),
    Register(String),
    BuiltInFunction(BuiltInFunctionsAst)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayLiteral {
    pub items : Vec<Expression>,
    pub cg_items : Vec<CgExpression>
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Number(i64),
    Array(ArrayLiteral),

    // Is used only for erroring (for type_string func)
    StringEmpty,
    NumberEmpty,
    ArrayEmpty
}

impl Literal {
    pub fn to_string(&self) -> String {
        let res : String = match self {
            Literal::String(str) => str.clone(),
            Literal::Number(num) => num.to_string(),
            _ => unreachable!()
        };

        return res;
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayType {
    pub item_count : usize,
    pub variable_type : Box<VariableType>
}

impl ArrayType {
    pub fn none() -> Self {
        return Self { item_count: 0, variable_type: Box::new(VariableType::Void) }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VariableType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Void,
    Array(ArrayType),
    ArrayEmpty,
}

#[macro_export]
macro_rules! num_types {
    () => {
        VariableType::I8 | VariableType::I16 | VariableType::I32 | VariableType::I64 | VariableType::U8 | VariableType::U16 | VariableType::U32 | VariableType::U64
    };
}

impl VariableType {
    #[inline]
    pub fn get_variable_size(&self) -> usize {
        return match self {
            VariableType::I8 => 1,
            VariableType::I16 => 2,
            VariableType::I32 => 4,
            VariableType::I64 => 8,
            VariableType::U8 => 1,
            VariableType::U16 => 2,
            VariableType::U32 => 4,
            VariableType::U64 => 8,
            VariableType::Array(arr) => arr.item_count * arr.variable_type.get_variable_size(),
            VariableType::Void => 0,
            VariableType::ArrayEmpty => 0
        }
    }
}

// Code gen specific Structs
#[derive(Debug, PartialEq, Clone)]
pub struct CgStatement {
    pub statement_type : CgStatementType
}

#[derive(Debug, PartialEq, Clone)]
pub enum CgStatementType {
    Compare(CgCompare),
    VariableAssignment(CgVariableAssignment),
    BuiltInFunction(CgBuiltInFunctions)
}

#[derive(Debug, PartialEq, Clone)]
pub struct CgCompare {
    pub conditions : Vec<CmpCondition>,
    pub expressions : [CgExpression; 2],
    pub new_exit_label : String
}

#[derive(Debug, PartialEq, Clone)]
pub enum CgBuiltInFunctions {
    Assembly(String),
    BranchLinked(CgBranchLinked),
    Branch(CgBranch)
}

#[derive(Debug, PartialEq, Clone)]
pub struct CgBranch {
    pub branch_name : String
}

#[derive(Debug, PartialEq, Clone)]
pub struct CgBranchLinked {
    pub function_name : String,
    pub args : Vec<CgExpression>
}

#[derive(Debug, PartialEq, Clone)]
pub enum CgExpression {
    Identifier(CgIdentifiers),
    Literal(Literal)
}

#[derive(Debug, PartialEq, Clone)]
pub enum CgIdentifiers {
    StackVariableData(StackVariableData),
    ArrayVariableData(ArrayVariableData)
}

#[derive(Debug, PartialEq, Clone)]
pub struct StackVariableData {
    pub offset : usize,
    pub variable_type : VariableType
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayVariableData {
    pub arr_index : Box<CgExpression>,
    pub variable_type : VariableType,
    pub offset : usize
}

#[derive(Debug, PartialEq, Clone)]
pub struct CgVariableAssignment {
    pub assign_value : CgExpression,
    pub assign_type : CgVariableAssignmentType,
    pub variable_type : VariableType
}

#[derive(Debug, PartialEq, Clone)]
pub enum CgVariableAssignmentType {
    CompileTimeStackOffset(usize),
    ArrayItem(ArrayVariableData),
    Register(String)
}

#[derive(Debug, PartialEq, Clone)]
pub enum CmpConditionType {
    Equal,
    NotEqual
}

#[derive(Debug, PartialEq, Clone)]
pub struct CmpCondition {
    pub condition_type : CmpConditionType,
    pub body : Vec<Statement>,
    pub scope : usize
}

impl CmpCondition {
    pub fn new(condition_type : CmpConditionType) -> Self {
        return Self{condition_type, body: Vec::new(), scope: 0};
    }

    pub fn set_body(&mut self, body : Vec<Statement>) -> () {
        self.body = body;
    }

    pub fn to_string(&self) -> String {
        let res = match self.condition_type {
            CmpConditionType::Equal => "eq",
            CmpConditionType::NotEqual => "ne"
        };

        return String::from(res);
    }

    pub fn from_token(input : &Token) -> Option<Self> {
        if let TokenType::Identifier(Identifiers::Identifier(identifier)) = input.kind.clone() {
            let cmp_type : CmpConditionType = match identifier.as_str() {
                "eq" => {
                    CmpConditionType::Equal
                },
                "ne" => {
                    CmpConditionType::NotEqual
                },
                _ => {
                    return None;
                }
            };

            return Some(CmpCondition::new(cmp_type));
        }

        return None;
    }
}

// ***************************** //
// *TYPE_STRING_IMPLEMENTATIONS* //
// ***************************** //

impl VariableType {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            VariableType::I8 => "i8",
            VariableType::I16 => "i16",
            VariableType::I32 => "i32",
            VariableType::I64 => "i64",
            VariableType::U8 => "u8",
            VariableType::U16 => "u16",
            VariableType::U32 => "u32",
            VariableType::U64 => "u64",
            VariableType::Void => "void",
            VariableType::Array(child) => {
                &format!("array[{}, {}]", child.variable_type.type_string(), child.item_count)
            },
            VariableType::ArrayEmpty => "array"
        };

        return String::from(str);
    }
}

impl Statements {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Statements::Assignment(_) => "Variable assignment",
            Statements::VariableDeclaration(_) => "Variable declaration",
            Statements::FunctionDeclaration(_) => "Function declaration",
            Statements::Compare(_) => "Compare function",
            Statements::StackFramePop => "Stack frame pop",
            Statements::Expression(expression) => &expression.type_string(),

            Statements::AssignmentEmpty => "Variable assignment",
            Statements::VariableDeclarationEmpty => "Variable declaration",
            Statements::FunctionDeclarationEmpty => "Function declaration",
            Statements::CompareEmpty => "Compare function",
            Statements::ExpressionEmpty => "Expression",

        };

        return String::from(str);
    }
}

impl BuiltInFunctionsAst {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            BuiltInFunctionsAst::StackOffset(_) => "Built in function Stack Offset",
            BuiltInFunctionsAst::Format(_) => "Built in function Format",
            BuiltInFunctionsAst::Assembly(_) => "Built in function Assembly",
            BuiltInFunctionsAst::BranchLinked(_) => "Built in function Branch Linked",
            BuiltInFunctionsAst::StackOffsetEmpty => "Built in function Stack Offset",
            BuiltInFunctionsAst::FormatEmpty => "Built in function Format",
            BuiltInFunctionsAst::AssemblyEmpty => "Built in function Assembly",
            BuiltInFunctionsAst::BranchLinkedEmpty => "Built in function Branch Linked"

        };

        return String::from(str);
    }
}

impl Expression {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Expression::ArrayIndex(array_index) => {
                &format!("ArrayIndex[{}]", array_index.index.type_string())
            },
            Expression::Register(register) => {
                &format!("Register[{}]", register)
            },
            Expression::Literal(literal) => &literal.type_string(),
            Expression::Identifier(identifier) => &identifier.type_string(),
            Expression::BuiltInFunction(built_in_function) => &built_in_function.type_string()
        };

        return String::from(str);
    }
}

impl Literal {
    pub fn type_string(&self) -> String {
        let str : &str = match self {
            Literal::String(_) => "String literal",
            Literal::Number(_) => "Number literal",
            Literal::Array(_) => "Array literal",
            Literal::StringEmpty => "String literal",
            Literal::NumberEmpty => "Number literal",
            Literal::ArrayEmpty => "Array literal"
        };

        return String::from(str);
    }
}
