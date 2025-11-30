use std::ops::Index;

use crate::{aliases::Result, runtime::values::VmValue};

use super::{
    byte_reader::ByteReader,
    type_table::{PrimitiveType, Type, TypeId, TypeTable},
};

#[derive(Debug, Copy, Clone)]
pub struct ConstIndex(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct StringEntry {
    offset: usize,
    len: usize,
}

#[derive(Debug)]
pub struct ConstPool<'const_data> {
    consts: Vec<VmValue>,
    strings: Vec<StringEntry>,
    const_data: &'const_data [u8],
}

impl<'const_data> ConstPool<'const_data> {
    pub fn get_string(&self, string_id: usize) -> Option<&str> {
        let string_entry = self.strings.get(string_id)?;
        let string_bytes = &self.const_data[string_entry.offset..string_entry.offset + string_entry.len];
        std::str::from_utf8(string_bytes).ok()
    }

    pub fn parse(const_data: &'const_data [u8], type_table: &TypeTable) -> Result<Self> {
        let mut consts = vec![];
        let mut strings = vec![];
        let mut reader = ByteReader::new(const_data, const_data.len());

        while let Ok(id) = reader.read_safe::<TypeId>() {
            let ty = &type_table[id];
            match ty {
                Type::Primitive(PrimitiveType::Str, _) => {
                    let len = reader.read::<u64>()? as usize;
                    let offset = reader.position();
                    reader.skip(len);
                    let id = strings.len();
                    strings.push(StringEntry { len, offset });
                    consts.push(VmValue::const_string(id));
                }
                Type::Primitive(t, _) => {
                    let v = match t {
                        PrimitiveType::Int => VmValue::Int(reader.read()?),
                        PrimitiveType::Bool => VmValue::Bool(reader.read()?),
                        PrimitiveType::Float => VmValue::Float(reader.read()?),
                        PrimitiveType::Str => unreachable!(),
                    };
                    consts.push(v);
                }
                Type::Void => {
                    consts.push(VmValue::Int(0));
                }
                _ => (),
            }
        }

        Ok(Self {
            consts,
            strings,
            const_data,
        })
    }
}

impl Index<ConstIndex> for ConstPool<'_> {
    type Output = VmValue;

    fn index(&self, index: ConstIndex) -> &Self::Output {
        &self.consts[index.0]
    }
}
