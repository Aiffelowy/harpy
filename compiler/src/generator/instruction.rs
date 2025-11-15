use crate::semantic_analyzer::{
    const_pool::ConstIndex, function_table::FuncIndex, type_table::RuntimeTypeIndex,
};

macro_rules! impl_extend {
    ($type:ty, $size:literal) => {
        impl IntoIterator for $type {
            type Item = u8;
            type IntoIter = std::array::IntoIter<u8, $size>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.to_be_bytes().into_iter()
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Hash, Eq)]
pub struct LocalAddress(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Hash, Eq)]
pub struct CodeAddress(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub u64);

impl_extend!(LocalAddress, 2);
impl_extend!(FuncIndex, 4);
impl_extend!(ConstIndex, 4);
impl_extend!(RuntimeTypeIndex, 4);
impl_extend!(CodeAddress, 8);

macro_rules! define_instruction_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $( ($param:ty) )? = $opcode:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $( ($param) )? = $opcode
            ),*
        }

        impl $name {
            pub fn opcode(&self) -> u8 {
                unsafe { *(self as *const Self as *const u8) }
            }
        }
    };

    (@ignore $t:tt) => {}
}

define_instruction_enum!(
    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[repr(u8)]
    pub enum Instruction {
        NOP = 0x0,

        LOAD_CONST(ConstIndex) = 0x01,

        PUSH_ADDR_LOCAL(LocalAddress) = 0x10,
        LOAD_LOCAL(LocalAddress) = 0x11,
        STORE_LOCAL(LocalAddress) = 0x12,

        LOAD = 0x31,
        STORE = 0x32,

        BOX_ALLOC(RuntimeTypeIndex) = 0x40,

        ADD = 0x50,
        SUB = 0x51,
        MUL = 0x52,
        DIV = 0x53,
        NEG = 0x54,
        INC = 0x55,

        JMP(Label) = 0x60,
        JMP_IF_TRUE(Label) = 0x61,
        JMP_IF_FALSE(Label) = 0x62,

        CALL(FuncIndex) = 0x70,
        RET = 0x71,

        EQ = 0x80,
        NEQ = 0x81,
        LT = 0x82,
        LTE = 0x83,
        GT = 0x84,
        GTE = 0x85,
        AND = 0x86,
        OR = 0x87,
        NOT = 0x88,

        POP = 0x90,
        DUP = 0x91,

        HALT = 0xFF,
    }
);
