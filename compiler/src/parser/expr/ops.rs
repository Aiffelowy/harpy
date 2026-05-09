#[derive(Debug, Clone, Copy)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    And,
    Or,
    Gt,
    Lt,
    Eq,
    GtEq,
    LtEq,
    Neq,
}

#[derive(Debug, Clone, Copy)]
pub enum PrefixOp {
    Minus,
    Plus,
    Neg,
}

#[derive(Debug, Clone, Copy)]
pub enum AssignOp {
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
