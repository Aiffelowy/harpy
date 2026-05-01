use crate::{
    aliases::Result,
    lexer::tokens::Ident,
    parse_separated,
    parser::{expr::expr_defs::Expr, Node, Parser},
    t, tt,
};

#[derive(Debug, Clone)]
pub enum BaseType {
    Int,
    Float,
    Bool,
    Str,
    Custom(Ident),
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub args: Vec<Node<Type>>,
    pub return_type: Box<Node<Type>>,
}

#[derive(Debug, Clone)]
pub enum TypeInner {
    Boxed(Box<Node<TypeInner>>),
    Ref(Box<Node<TypeInner>>),
    Array(Box<Node<TypeInner>>, Option<Box<Node<Expr>>>),
    FunctionType(FunctionType),
    Base(Node<BaseType>),
    Void,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub mutable: bool,
    pub inner: TypeInner,
}

impl Type {
    pub fn void() -> Self {
        Self {
            mutable: false,
            inner: TypeInner::Void,
        }
    }
}

impl<'parser> Parser<'parser> {
    fn parse_base_type(&mut self) -> Result<BaseType> {
        let base = match self.peek()? {
            tt!(int) => {
                self.consume::<t!(int)>()?;
                BaseType::Int
            }
            tt!(float) => {
                self.consume::<t!(float)>()?;
                BaseType::Float
            }
            tt!(str) => {
                self.consume::<t!(str)>()?;
                BaseType::Str
            }
            tt!(bool) => {
                self.consume::<t!(bool)>()?;
                BaseType::Bool
            }

            _ => BaseType::Custom(self.consume()?),
        };

        Ok(base)
    }

    fn parse_function_type(&mut self) -> Result<FunctionType> {
        self.consume::<t!(fn)>()?;
        let args = parse_separated!(self, "(", ")",,, self.parse_node(Self::parse_type)?);
        let return_type = Box::new(self.parse_node(Self::parse_function_return_type)?);
        Ok(FunctionType { args, return_type })
    }

    fn parse_inner_type(&mut self) -> Result<TypeInner> {
        let inner = match self.peek()? {
            tt!(ref) => {
                self.consume::<t!(ref)>()?;
                TypeInner::Ref(Box::new(self.parse_node(Self::parse_inner_type)?))
            }
            tt!(boxed) => {
                self.consume::<t!(boxed)>()?;
                TypeInner::Boxed(Box::new(self.parse_node(Self::parse_inner_type)?))
            }
            tt!(.) => {
                self.consume::<t!(.)>()?;
                TypeInner::Unknown
            }
            tt!("[") => {
                self.consume::<t!("[")>()?;
                let inner_type = Box::new(self.parse_node(Self::parse_inner_type)?);
                let mut size = None;
                if let tt!(:) = self.peek()? {
                    self.consume::<t!(:)>()?;
                    size = Some(Box::new(self.parse_node(Self::parse_expr)?));
                }
                let inner = TypeInner::Array(inner_type, size);
                self.consume::<t!("]")>()?;
                inner
            }
            tt!(fn) => {
                let inner = self.parse_function_type()?;
                TypeInner::FunctionType(inner)
            }
            _ => TypeInner::Base(self.parse_node(Self::parse_base_type)?),
        };

        Ok(inner)
    }

    pub fn parse_type(&mut self) -> Result<Type> {
        let mutable = if let tt!(mut) = self.peek()? {
            self.consume::<t!(mut)>()?;
            true
        } else {
            false
        };

        let inner = self.parse_inner_type()?;

        Ok(Type { inner, mutable })
    }
}
