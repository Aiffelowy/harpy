use crate::{
    aliases::Result,
    lexer::tokens::Ident,
    parse_separated,
    parser::{expr::expr_defs::Expr, Node, Parser},
    peek_and_consume, t, tt,
};

#[derive(Debug, Clone)]
pub enum Type {
    Boxed(Box<Node<Type>>, bool),
    Ref(Box<Node<Type>>, bool),
    Array(Box<Node<Type>>, Option<Box<Node<Expr>>>),
    FunctionType(FunctionType),
    Int,
    Float,
    Bool,
    Str,
    Custom(Ident),
    Void,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub args: Vec<Node<Type>>,
    pub return_type: Box<Node<Type>>,
}

impl Type {
    pub fn void() -> Self {
        Self::Void
    }
}

impl<'parser> Parser<'parser> {
    fn parse_function_type(&mut self) -> Result<FunctionType> {
        self.consume::<t!(fn)>()?;
        let args = parse_separated!(self, "(", ")",,, self.parse_node(Self::parse_type)?);
        let return_type = Box::new(self.parse_node(Self::parse_function_return_type)?);
        Ok(FunctionType { args, return_type })
    }

    fn parse_inner_type(&mut self, allow_infer: bool) -> Result<Type> {
        let inner = match self.peek()? {
            tt!(int) => {
                self.consume::<t!(int)>()?;
                Type::Int
            }
            tt!(float) => {
                self.consume::<t!(float)>()?;
                Type::Float
            }
            tt!(str) => {
                self.consume::<t!(str)>()?;
                Type::Str
            }
            tt!(bool) => {
                self.consume::<t!(bool)>()?;
                Type::Bool
            }
            tt!(boxed) => {
                self.consume::<t!(boxed)>()?;
                let mutable = peek_and_consume!(self, mut);
                Type::Boxed(
                    Box::new(self.parse_node(|p| p.parse_inner_type(allow_infer))?),
                    mutable,
                )
            }
            tt!(ref) => {
                self.consume::<t!(ref)>()?;
                let mutable = peek_and_consume!(self, mut);
                Type::Ref(
                    Box::new(self.parse_node(|p| p.parse_inner_type(allow_infer))?),
                    mutable,
                )
            }
            tt!(.) if allow_infer => {
                self.consume::<t!(.)>()?;
                Type::Unknown
            }
            tt!("[") => {
                self.consume::<t!("[")>()?;
                let inner_type = Box::new(self.parse_node(|p| p.parse_inner_type(allow_infer))?);
                let mut size = None;
                if let tt!(:) = self.peek()? {
                    self.consume::<t!(:)>()?;
                    size = Some(Box::new(self.parse_node(Self::parse_expr)?));
                }
                let inner = Type::Array(inner_type, size);
                self.consume::<t!("]")>()?;
                inner
            }
            tt!(fn) => {
                let inner = self.parse_function_type()?;
                Type::FunctionType(inner)
            }
            tt!(void) => {
                self.consume::<t!(void)>()?;
                Type::Void
            }

            _ => Type::Custom(self.consume()?),
        };

        Ok(inner)
    }

    pub(in crate::parser) fn parse_type_with_infer(&mut self) -> Result<Type> {
        return self.parse_inner_type(true);
    }

    pub(in crate::parser) fn parse_type(&mut self) -> Result<Type> {
        return self.parse_inner_type(false);
    }
}
