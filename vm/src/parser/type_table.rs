use std::ops::Index;

use crate::{
    aliases::Result,
    err::ParseError,
    runtime::values::{HeapAddress, VmValue},
};

use super::byte_reader::{ByteReader, ReadSafe};

#[derive(Debug, Clone, Copy)]
pub struct TypeId(pub usize);

impl ReadSafe for TypeId {
    fn read_safe(reader: &mut ByteReader) -> Result<Self> {
        Ok(Self(reader.read::<u32>()? as usize))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PrimitiveType {
    Int = 0x1,
    Float = 0x2,
    Str = 0x3,
    Bool = 0x4,
}

impl ReadSafe for PrimitiveType {
    fn read_safe(reader: &mut ByteReader) -> Result<Self> {
        Ok(match reader.read::<u8>()? {
            1 => Self::Int,
            2 => Self::Float,
            3 => Self::Str,
            4 => Self::Bool,
            _ => return Err(ParseError::UnknownTypeId.into()),
        })
    }
}

impl PrimitiveType {
    pub fn construct(&self, reader: &mut ByteReader) -> Result<VmValue> {
        Ok(match self {
            Self::Int => VmValue::Int(reader.read()?),
            Self::Float => VmValue::Float(reader.read()?),
            Self::Bool => VmValue::Bool(reader.read()?),
            Self::Str => VmValue::StringHandle {
                len: reader.read()?,
                ptr: reader.read_safe()?,
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeSize(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct Pointee(pub usize);

#[derive(Debug)]
pub enum Type {
    Void,
    Primitive(PrimitiveType, TypeSize),
    Pointer(Pointee),
    Ref(Pointee),
    Custom(TypeSize),
}

impl Type {
    pub fn size(&self) -> TypeSize {
        match self {
            Self::Void => TypeSize(0),
            Self::Primitive(_, size) => *size,
            Self::Pointer(_) => TypeSize(16),
            Self::Ref(_) => TypeSize(16),
            Self::Custom(size) => *size,
        }
    }
}

impl Type {
    pub fn construct(&self, reader: &mut ByteReader) -> Result<VmValue> {
        let v = match self {
            Type::Void => VmValue::Int(0),
            Type::Ref(_) => VmValue::Ref(reader.read_safe()?, TypeId(reader.read()?)),
            Type::Primitive(p, _) => p.construct(reader)?,
            Type::Pointer(_) => {
                VmValue::Pointer(HeapAddress(reader.read()?), TypeId(reader.read()?))
            }
            Type::Custom(s) => {
                reader.skip(s.0);
                VmValue::Int(0)
            }
        };

        Ok(v)
    }
}

#[derive(Debug)]
pub struct TypeTable {
    tt: Vec<Type>,
}

impl TypeTable {
    pub fn parse(mut reader: ByteReader) -> Result<Self> {
        let mut tt = vec![];

        while let Ok(id) = reader.read::<u8>() {
            tt.push(match id {
                0 => Type::Void,
                1 => Type::Primitive(reader.read_safe()?, TypeSize(reader.read::<u8>()? as usize)),
                2 => Type::Pointer(Pointee(reader.read::<u32>()? as usize)),
                3 => Type::Ref(Pointee(reader.read::<u32>()? as usize)),
                4 => Type::Custom(TypeSize(reader.read::<u8>()? as usize)),
                _ => return Err(crate::err::ParseError::UnknownTypeId.into()),
            });
        }
        Ok(TypeTable { tt })
    }
}

impl Index<TypeId> for TypeTable {
    type Output = Type;

    fn index(&self, index: TypeId) -> &Self::Output {
        unsafe { self.tt.get_unchecked(index.0) }
    }
}
