use crate::{aliases::Result, runtime::runtime::Runtime};

use super::{
    byte_reader::ByteReader,
    const_pool::ConstPool,
    function_table::{FunctionIndex, FunctionTable},
    type_table::TypeTable,
};

#[derive(Debug)]
pub struct Header {
    magic_number: [u8; 5],
    version: u16,
    flags: u16,
    pub entry_point: FunctionIndex,
    type_table_offset: u32,
    const_pool_offset: u32,
    function_table_offset: u32,
    bytecode_offset: u32,
    bytecode_size: u32,
}

pub const HEADER_SIZE: usize = std::mem::size_of::<Header>() + 5;

impl Header {
    pub fn parse(mut bytes: ByteReader) -> Result<Self> {
        Ok(Self {
            magic_number: bytes.read()?,
            version: bytes.read()?,
            flags: bytes.read()?,
            entry_point: FunctionIndex(bytes.read::<u32>()? as usize),
            type_table_offset: bytes.read()?,
            const_pool_offset: bytes.read()?,
            function_table_offset: bytes.read()?,
            bytecode_offset: bytes.read()?,
            bytecode_size: bytes.read()?,
        })
    }

    pub fn into_runtime<'split>(self, bytecode: &'split [u8]) -> Result<Runtime<'split>> {
        let tt = TypeTable::parse(ByteReader::new(
            &bytecode[self.type_table_offset as usize..self.const_pool_offset as usize],
            (self.const_pool_offset - self.type_table_offset) as usize,
        ))?;

        let cp = ConstPool::parse(
            &bytecode[self.const_pool_offset as usize..self.function_table_offset as usize],
            &tt,
        )?;
        let ft = FunctionTable::parse(
            ByteReader::new(
                &bytecode[self.function_table_offset as usize..self.bytecode_offset as usize],
                (self.bytecode_offset - self.function_table_offset) as usize,
            ),
            &tt,
        )?;

        let bc = &bytecode
            [self.bytecode_offset as usize..(self.bytecode_offset + self.bytecode_size) as usize];

        Runtime::new(self, tt, cp, ft, bc)
    }
}
