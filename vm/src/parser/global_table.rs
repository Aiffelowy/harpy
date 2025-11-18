use std::ops::Index;

use crate::aliases::Result;
use crate::runtime::values::VmValue;

use super::{
    byte_reader::ByteReader,
    type_table::{TypeId, TypeTable},
};

#[derive(Debug, Clone, Copy)]
pub struct GlobalIndex(pub usize);

#[derive(Debug)]
pub struct GlobalInfo {
    pub type_id: TypeId,
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug)]
pub struct GlobalTable {
    global_infos: Vec<GlobalInfo>,
    global_memory: Vec<u8>,
    total_size: usize,
}

impl GlobalTable {
    pub fn parse(mut reader: ByteReader, type_table: &TypeTable) -> Result<Self> {
        let mut global_infos = vec![];
        let mut total_size = 0;

        while let Ok(type_id) = reader.read_safe::<TypeId>() {
            let type_info = &type_table[type_id];
            let size = type_info.size().0;
            
            global_infos.push(GlobalInfo {
                type_id,
                offset: total_size,
                size,
            });
            
            total_size += size;
        }

        let global_memory = vec![0u8; total_size];

        Ok(Self {
            global_infos,
            global_memory,
            total_size,
        })
    }

    pub fn read_global(&self, global_id: GlobalIndex, type_table: &TypeTable) -> Result<VmValue> {
        let global_info = &self.global_infos[global_id.0];
        let type_info = &type_table[global_info.type_id];
        
        let mut reader = ByteReader::new(&self.global_memory[global_info.offset..], type_info.size().0);
        type_info.construct(&mut reader)
    }

    pub fn write_global(&mut self, global_id: GlobalIndex, value: VmValue) {
        let global_info = &self.global_infos[global_id.0];
        let memory_slice = &mut self.global_memory[global_info.offset..global_info.offset + global_info.size];
        value.write_bytes(memory_slice);
    }
}

impl Index<GlobalIndex> for GlobalTable {
    type Output = GlobalInfo;

    fn index(&self, index: GlobalIndex) -> &Self::Output {
        &self.global_infos[index.0]
    }
}