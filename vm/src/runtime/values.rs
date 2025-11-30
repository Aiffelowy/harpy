use crate::{
    aliases::Result,
    err::RuntimeError,
    parser::{byte_reader::ReadSafe, type_table::TypeId, const_pool::ConstPool},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapAddress(pub usize);

impl ReadSafe for HeapAddress {
    fn read_safe(
        reader: &mut crate::parser::byte_reader::ByteReader,
    ) -> crate::aliases::Result<Self> {
        Ok(Self(reader.read()?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackAddress(pub usize);

impl ReadSafe for StackAddress {
    fn read_safe(reader: &mut crate::parser::byte_reader::ByteReader) -> Result<Self> {
        Ok(Self(reader.read()?))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VmValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    StringHandle { len: usize, ptr: HeapAddress },
    Pointer(HeapAddress, TypeId),
    Ref(StackAddress, TypeId),
}

macro_rules! arithmetic_op {
      ($name:ident, $op:tt) => {
          pub fn $name(self, other: VmValue) -> Result<VmValue> {
              use VmValue::*;
              Ok(match (self, other) {
                  (Int(a), Int(b)) => Int(a $op b),
                  (Float(a), Float(b)) => Float(a $op b),
                  _ => return Err(RuntimeError::InvalidOperation),
              })
          }
      };
  }

macro_rules! comparison_op {
      ($name:ident, $op:tt) => {
          pub fn $name(self, other: VmValue) -> Result<VmValue> {
              use VmValue::*;
              let result = match (self, other) {
                  (Int(a), Int(b)) => a $op b,
                  (Float(a), Float(b)) => a $op b,
                  _ => return Err(RuntimeError::InvalidOperation),
              };
              Ok(Bool(result))
          }
      };
  }

impl VmValue {
    pub fn write_bytes(self, memory: &mut [u8]) {
        match self {
            Self::Int(i) => memory.copy_from_slice(&i.to_be_bytes()),
            Self::Float(f) => memory.copy_from_slice(&f.to_be_bytes()),
            Self::Bool(b) => memory[0] = b as u8,
            Self::StringHandle { len, ptr } => {
                memory[0..8].copy_from_slice(&len.to_be_bytes());
                memory[8..16].copy_from_slice(&ptr.0.to_be_bytes());
            }
            Self::Pointer(address, ti) => {
                memory[0..8].copy_from_slice(&address.0.to_be_bytes());
                memory[8..16].copy_from_slice(&ti.0.to_be_bytes());
            }
            Self::Ref(address, ti) => {
                memory[0..8].copy_from_slice(&address.0.to_be_bytes());
                memory[8..16].copy_from_slice(&ti.0.to_be_bytes());
            }
        }
    }

    pub fn const_string(stringid: usize) -> Self {
        VmValue::StringHandle {
            len: stringid,
            ptr: HeapAddress(0),
        }
    }

    pub fn heap_string(len: usize, ptr: HeapAddress) -> Self {
        VmValue::StringHandle { len, ptr }
    }

    arithmetic_op!(add, +);
    arithmetic_op!(sub, -);
    arithmetic_op!(mul, *);
    arithmetic_op!(div, /);
    arithmetic_op!(modulo, %);

    pub fn neg(self) -> Result<VmValue> {
        match self {
            Self::Int(i) => Ok(VmValue::Int(-i)),
            Self::Float(f) => Ok(VmValue::Float(-f)),
            _ => Err(RuntimeError::InvalidOperation),
        }
    }

    pub fn inc(self) -> Result<VmValue> {
        match self {
            Self::Int(i) => Ok(VmValue::Int(i + 1)),
            Self::Float(f) => Ok(VmValue::Float(f + 1.)),
            _ => Err(RuntimeError::InvalidOperation),
        }
    }

    pub fn as_bool(self) -> Result<bool> {
        if let Self::Bool(b) = self {
            return Ok(b);
        }

        Err(RuntimeError::InvalidOperation)
    }

    pub fn and(self, other: VmValue) -> Result<VmValue> {
        use VmValue::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(a && b)),
            _ => Err(RuntimeError::InvalidOperation),
        }
    }

    pub fn or(self, other: VmValue) -> Result<VmValue> {
        use VmValue::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(a || b)),
            _ => Err(RuntimeError::InvalidOperation),
        }
    }

    pub fn not(self) -> Result<VmValue> {
        use VmValue::*;
        match self {
            Bool(b) => Ok(Bool(!b)),
            _ => Err(RuntimeError::InvalidOperation),
        }
    }

    comparison_op!(lt, <);
    comparison_op!(le, <=);
    comparison_op!(gt, >);
    comparison_op!(ge, >=);

    pub fn eq(self, other: VmValue) -> Result<VmValue> {
        use VmValue::*;
        let result = match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            _ => return Err(RuntimeError::InvalidOperation),
        };
        Ok(Bool(result))
    }

    pub fn ne(self, other: VmValue) -> Result<VmValue> {
        Ok(Self::Bool(!self.eq(other)?.as_bool()?))
    }

    pub fn display_with_const_pool(&self, const_pool: &ConstPool) -> String {
        match self {
            VmValue::Int(i) => i.to_string(),
            VmValue::Float(f) => f.to_string(),
            VmValue::Bool(b) => b.to_string(),
            VmValue::StringHandle { len, ptr } => {
                if ptr.0 == 0 {
                    // This is a const pool string
                    const_pool.get_string(*len)
                        .map(|s| format!("\"{}\"", s))
                        .unwrap_or_else(|| format!("StringHandle {{ len: {}, ptr: {:?} }}", len, ptr))
                } else {
                    // This would be a heap string (not implemented)
                    format!("StringHandle {{ len: {}, ptr: {:?} }}", len, ptr)
                }
            }
            VmValue::Pointer(addr, type_id) => format!("Pointer({:?}, {:?})", addr, type_id),
            VmValue::Ref(addr, type_id) => format!("Ref({:?}, {:?})", addr, type_id),
        }
    }
}
