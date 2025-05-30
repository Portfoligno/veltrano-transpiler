#[derive(Debug, Clone)]
pub struct Type {
    pub base: BaseType,
    pub reference_depth: u32,
}

impl Type {
    pub fn owned(base: BaseType) -> Self {
        Type {
            base,
            reference_depth: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BaseType {
    Int,
    Bool,
    Unit,
    Nothing,
    Str,
    String,
    MutRef(Box<Type>),
    Box(Box<Type>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Identifier(String),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    FieldAccess(FieldAccessExpr),
}

#[derive(Debug, Clone)]
pub enum LiteralExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Unit,
    Null,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Minus,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: BinaryOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum Argument {
    Bare(Expr), // Just the expression: positional in function calls, field shorthand in struct init
    Named(String, Expr), // Explicit name: `name = value` → named parameter or struct field
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Box<Expr>,
    pub method: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<Expr>,
    pub field: String,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr, Option<(String, String)>), // Expression with optional inline comment (content, whitespace)
    VarDecl(VarDeclStmt, Option<(String, String)>), // Variable declaration with optional inline comment (content, whitespace)
    FunDecl(FunDeclStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(Option<Expr>, Option<(String, String)>), // Return statement with optional inline comment (content, whitespace)
    Block(Vec<Stmt>),
    Comment(CommentStmt),     // Standalone comments
    Import(ImportStmt),       // Import statement
    DataClass(DataClassStmt), // Data class declaration
}

#[derive(Debug, Clone)]
pub struct CommentStmt {
    pub content: String,
    pub is_block_comment: bool,
    pub preceding_whitespace: String,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunDeclStmt {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub type_name: String,
    pub method_name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DataClassStmt {
    pub name: String,
    pub fields: Vec<DataClassField>,
}

#[derive(Debug, Clone)]
pub struct DataClassField {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
