use crate::{
    aliases::Result,
    lexer::tokens::Ident,
    parser::{Node, Parser},
    t, tt,
};

pub enum Primitive {
    Int,
    Float,
    Bool,
    Str,
}

pub enum BaseType {
    Base(Primitive),
    Custom(Ident),
}

pub struct FunctionType {
    pub args: Vec<Node<Type>>,
    pub return_type: Box<Node<Type>>,
}

pub enum TypeInner {
    Base(Node<BaseType>),
    Boxed(Box<Node<Type>>),
    Ref(Box<Node<Type>>),
    Array(Box<Node<Type>>),
    FunctionType(FunctionType),
    Void,
    Unknown,
}

pub struct Type {
    mutable: bool,
    inner: TypeInner,
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
                BaseType::Base(Primitive::Int)
            }
            tt!(float) => {
                self.consume::<t!(float)>()?;
                BaseType::Base(Primitive::Float)
            }
            tt!(str) => {
                self.consume::<t!(str)>()?;
                BaseType::Base(Primitive::Str)
            }
            tt!(bool) => {
                self.consume::<t!(bool)>()?;
                BaseType::Base(Primitive::Bool)
            }

            _ => BaseType::Custom(self.consume()?),
        };

        Ok(base)
    }

    fn parse_function_type(&mut self) -> Result<FunctionType> {
        self.consume::<t!(fn)>()?;
        self.consume::<t!("(")>()?;
        let mut args = vec![];
        loop {
            if let tt!(")") | tt!(eof) = self.peek()? {
                break;
            }

            args.push(self.parse_node(Self::parse_type)?);
            if let tt!(,) = self.peek()? {
                self.consume::<t!(,)>()?;
            } else {
                break;
            }
        }

        self.consume::<t!(")")>()?;
        let return_type = Box::new(self.parse_node(Self::parse_function_return_type)?);
        Ok(FunctionType { args, return_type })
    }

    pub fn parse_type(&mut self) -> Result<Type> {
        let mutable = if let tt!(mut) = self.peek()? {
            self.consume::<t!(mut)>()?;
            true
        } else {
            false
        };

        let inner = match self.peek()? {
            tt!(ref) => {
                self.consume::<t!(ref)>()?;
                TypeInner::Ref(Box::new(self.parse_node(Self::parse_type)?))
            }
            tt!(boxed) => {
                self.consume::<t!(boxed)>()?;
                TypeInner::Boxed(Box::new(self.parse_node(Self::parse_type)?))
            }
            tt!(.) => {
                self.consume::<t!(.)>()?;
                TypeInner::Unknown
            }
            tt!("[") => {
                self.consume::<t!("[")>()?;
                let inner = TypeInner::Array(Box::new(self.parse_node(Self::parse_type)?));
                self.consume::<t!("]")>()?;
                inner
            }
            tt!(fn) => {
                let inner = self.parse_function_type()?;
                TypeInner::FunctionType(inner)
            }
            _ => TypeInner::Base(self.parse_node(Self::parse_base_type)?),
        };

        Ok(Type { inner, mutable })
    }
}
