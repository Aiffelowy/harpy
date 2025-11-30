use crate::{aliases::{Result, MAGIC_NUMBER, VERSION}, err::ParseError, runtime::runtime::Runtime};

use super::{
    byte_reader::ByteReader,
    const_pool::ConstPool,
    function_table::{FunctionIndex, FunctionTable},
    global_table::GlobalTable,
    type_table::TypeTable,
};

#[derive(Debug)]
pub struct Header {
    #[allow(unused)]
    magic_number: [u8; 5],
    #[allow(unused)]
    version: u16,
    #[allow(unused)]
    flags: u16,
    pub main_index: FunctionIndex,
    type_table_offset: u32,
    global_table_offset: u32,
    const_pool_offset: u32,
    function_table_offset: u32,
    bytecode_offset: u32,
    bytecode_size: u32,
}

pub const HEADER_SIZE: usize = std::mem::size_of::<Header>() + 5;

impl Header {
    pub fn parse(mut bytes: ByteReader) -> Result<Self> {
        let magic_number = bytes.read()?;
        if magic_number != MAGIC_NUMBER {
            return Err(ParseError::InvalidFileType.into())
        }

        let version = bytes.read()?;
        if version != VERSION {
            return Err(ParseError::InvalidFileVersion.into());
        }

        Ok(Self {
            magic_number,
            version,
            flags: bytes.read()?,
            main_index: FunctionIndex(bytes.read::<u32>()? as usize),
            type_table_offset: bytes.read()?,
            global_table_offset: bytes.read()?,
            const_pool_offset: bytes.read()?,
            function_table_offset: bytes.read()?,
            bytecode_offset: bytes.read()?,
            bytecode_size: bytes.read()?,
        })
    }

    pub fn into_runtime<'split>(self, bytecode: &'split [u8]) -> Result<Runtime<'split>> {
        let tt = TypeTable::parse(ByteReader::new(
            &bytecode[self.type_table_offset as usize..self.global_table_offset as usize],
            (self.global_table_offset - self.type_table_offset) as usize,
        ))?;

        let gt = GlobalTable::parse(
            ByteReader::new(
                &bytecode[self.global_table_offset as usize..self.const_pool_offset as usize],
                (self.const_pool_offset - self.global_table_offset) as usize,
            ),
            &tt,
        )?;

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

        Runtime::new(self, tt, gt, cp, ft, bc)
    }
}
