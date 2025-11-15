use crate::aliases::Result;
use crate::err::RuntimeError;
use crate::parser::{
    byte_reader::{ByteReader, ReadSafe},
    const_pool::ConstIndex,
    function_table::{CodeAddress, FunctionIndex, LocalIndex},
    type_table::TypeId,
};

macro_rules! instructions {
      (
          $(
              $variant:ident $(( $($arg_name:ident : $arg_wrapper:ident < $arg_type:ty >),* ))? = $value:expr
          ),* $(,)?
      ) => {
          #[repr(u8)]
          #[allow(non_camel_case_types)]
          #[derive(Debug)]
          pub enum Instruction {
              $(
                  $variant $(( $($arg_wrapper),* ))? = $value
              ),*
          }

          impl ReadSafe for Instruction {
              fn read_safe(reader: &mut ByteReader) -> Result<Self> {
                  let opcode = reader.read::<u8>()?;

                  Ok(match opcode {
                      $(
                          $value => {
                              instructions!(@parse_args $variant reader $(($($arg_name : $arg_wrapper < $arg_type  >),*))?)
                          }
                      ),*
                      _ => return Err(RuntimeError::InvalidOpcode.into()),
                  })
              }
          }
      };

      (@parse_args $variant:ident $reader:ident) => {
          Instruction::$variant
      };

      (@parse_args $variant:ident $reader:ident ($($arg_name:ident : $arg_wrapper:ident < $arg_type:ty >),*)) => {
          {
              $(let $arg_name = $reader.read::<$arg_type>()?.try_into().unwrap();)*
              Instruction::$variant($($arg_wrapper($arg_name)),*)
          }
      };
  }

instructions!(
    NOP = 0x0,
    LOAD_CONST(id: ConstIndex<u32>) = 0x01,
    PUSH_ADDR_LOCAL(id: LocalIndex<u16>) = 0x10,
    LOAD_LOCAL(id: LocalIndex<u16>) = 0x11,
    STORE_LOCAL(id: LocalIndex<u16>) = 0x12,
    LOAD = 0x31,
    STORE = 0x32,
    BOX_ALLOC(id: TypeId<u32>) = 0x40,
    ADD = 0x50,
    SUB = 0x51,
    MUL = 0x52,
    DIV = 0x53,
    NEG = 0x54,
    INC = 0x55,
    JMP(ca: CodeAddress<u64>) = 0x60,
    JMP_IF_TRUE(ca: CodeAddress<u64>) = 0x61,
    JMP_IF_FALSE(ca: CodeAddress<u64>) = 0x62,
    CALL(fi: FunctionIndex<u32>) = 0x70,
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
);
