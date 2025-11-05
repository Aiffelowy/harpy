use super::BaseType;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RuntimeType {
    Base(BaseType),
    Boxed(Box<RuntimeType>),
    Ref(Box<RuntimeType>),
    Void,
}
