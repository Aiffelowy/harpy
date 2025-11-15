use std::ops::Index;

use crate::aliases::Result;

use super::{
    byte_reader::ByteReader,
    type_table::{TypeId, TypeTable},
};

#[derive(Debug, Clone, Copy)]
pub struct FunctionIndex(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct LocalIndex(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct CodeAddress(pub u64);

#[derive(Debug)]
pub struct FunctionInfo {
    pub code_offset: CodeAddress,
    pub param_count: usize,
    pub local_count: usize,
    pub local_types: Vec<TypeId>,

    pub stack_size: usize,
    pub local_offsets: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct FunctionTable {
    func_infos: Vec<FunctionInfo>,
}

impl FunctionTable {
    pub fn parse(mut reader: ByteReader, type_table: &TypeTable) -> Result<Self> {
        let mut func_infos = vec![];

        while let Ok(offset) = reader.read::<u64>() {
            let param_count = reader.read::<u16>()? as usize;
            let local_count: usize = reader.read::<u16>()? as usize;

            let mut local_types = Vec::with_capacity(local_count);
            let mut offsets = Vec::with_capacity(local_count);
            let mut stack_size: usize = 16;

            for _ in 0..local_count {
                let id = reader.read_safe()?;
                let size = type_table[id].size().0;
                offsets.push((stack_size, size));
                stack_size += size;
                local_types.push(id);
            }

            func_infos.push(FunctionInfo {
                code_offset: CodeAddress(offset),
                param_count,
                local_count,
                local_types,
                stack_size,
                local_offsets: offsets,
            });
        }

        Ok(Self { func_infos })
    }
}

impl Index<FunctionIndex> for FunctionTable {
    type Output = FunctionInfo;

    fn index(&self, index: FunctionIndex) -> &Self::Output {
        &self.func_infos[index.0]
    }
}
