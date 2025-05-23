#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Str,
    String,
    Bool,
    Unit,
    Ref(Box<Type>),
    Box(Box<Type>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Identifier(String),
    Binary(BinaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
}

#[derive(Debug, Clone)]
pub enum LiteralExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Null,
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
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Box<Expr>,
    pub method: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    VarDecl(VarDeclStmt),
    FunDecl(FunDeclStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(Option<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub name: String,
    pub is_mutable: bool,
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
pub struct Program {
    pub statements: Vec<Stmt>,
}
