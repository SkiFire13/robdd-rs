pub enum Expr {
    Var(Var),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Exists(Vec<Var>, Box<Expr>),
}

#[derive(Hash, PartialEq, Eq)]
pub struct Var(pub String);
