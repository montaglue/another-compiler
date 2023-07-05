#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: String) -> Self {
        Name { name }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    StringLiteral(String),
    Name(Name),
    Binary(Operator, Box<Expr>, Box<Expr>),
    Call(Name, Vec<Expr>),
}

impl Expr {
    #[allow(clippy::should_implement_trait)]
    pub fn add(left: Expr, right: Expr) -> Self {
        Expr::Binary(Operator::Add, Box::new(left), Box::new(right))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(left: Expr, right: Expr) -> Self {
        Expr::Binary(Operator::Sub, Box::new(left), Box::new(right))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn mul(left: Expr, right: Expr) -> Self {
        Expr::Binary(Operator::Mul, Box::new(left), Box::new(right))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn div(left: Expr, right: Expr) -> Self {
        Expr::Binary(Operator::Div, Box::new(left), Box::new(right))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Let(Name, Expr),
    Return(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    For(Box<Stmt>, Expr, Expr, Vec<Stmt>),
    Assign(Name, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Name,
    pub args: Vec<Name>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}
