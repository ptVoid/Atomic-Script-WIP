#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i32),
    Float(f32),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Tag(pub String);

pub fn get_operator_level(op: &str) -> u8 {
    match op {
        "&" | "|" => 1,
        "==" => 2,
        "<" | ">" => 3,
        "+" | "-" => 4,
        "*" | "/" => 5,
        _ => todo!(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    BinaryExpr(String, Box<Expr>, Box<Expr>),
    Ident(Ident),
    TaggedIdent(Tag, Ident),
    VarDeclare(Ident, Box<Expr>),
    VarAssign(Ident, Box<Expr>),
    // fn declare ast is genereated in a special Vec in Source
    FnCall(/* id */ Ident, /* args */ Vec<Expr>),
    IfExpr(
        /* condition */ Box<Expr>,
        /* body */ Vec<Expr>,
        /* alt */ Vec<Expr>,
    ),
    Block(Vec<Expr>),
    RetExpr(Box<Expr>),
}
