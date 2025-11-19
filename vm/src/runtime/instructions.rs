use crate::aliases::Result;
use crate::err::RuntimeError;

type InstructionFn = fn(
    &mut crate::runtime::runtime::Runtime<'_>,
) -> std::result::Result<(), crate::err::RuntimeError>;

macro_rules! instructions {
      (
          $(
              $variant:ident $(( $($arg_name:ident : $arg_wrapper:ident < $arg_type:ty >),* ))? = $value:expr => ($runtime:ident) => $execute:expr
          ),* $(,)?
      ) => {
          pub static EXECUTE_TABLE: [InstructionFn; 256] = {
              let mut table = [invalid_execute as InstructionFn; 256];
              $(
                  table[$value] = $variant::execute;
              )*
              table
          };

          fn invalid_execute(_runtime: &mut $crate::runtime::runtime::Runtime) -> Result<()> {
              Err(RuntimeError::InvalidOpcode.into())
          }

          $(
              #[allow(non_snake_case)]
              #[allow(unused_imports)]
              mod $variant {
                  use $crate::aliases::Result;
                  use $crate::err::RuntimeError;
                  use $crate::ByteReader;
                  use $crate::parser::global_table::GlobalIndex;
                  use $crate::parser::type_table::TypeId;
                  use $crate::parser::function_table::LocalIndex;
                  use $crate::parser::function_table::CodeAddress;
                  use $crate::parser::const_pool::ConstIndex;
                  use $crate::parser::function_table::FunctionIndex;
                  #[allow(unused)]
                  pub(super) fn execute($runtime: &mut $crate::runtime::runtime::Runtime) -> Result<()> {
                      $($(let $arg_name = $runtime.bytecode.read::<$arg_type>()?.try_into().unwrap();)*)?
                      $execute
                  }
              }
          )*
      };
  }

instructions!(
    NOP = 0x0 => (rt) => Ok(()),
    LOAD_CONST(id: ConstIndex<u32>) = 0x01 => (rt) => {
        rt.load_const(ConstIndex(id));
        Ok(())
    },
    PUSH_ADDR_LOCAL(id: LocalIndex<u16>) = 0x10 => (rt) => {
        rt.push_addr_local(LocalIndex(id));
        Ok(())
    },
    LOAD_LOCAL(id: LocalIndex<u16>) = 0x11 => (rt) => rt.load_local(LocalIndex(id)),
    STORE_LOCAL(id: LocalIndex<u16>) = 0x12 => (rt) => rt.store_local(LocalIndex(id)),
    LOAD_GLOBAL(id: GlobalIndex<u16>) = 0x13 => (rt) => rt.load_global(GlobalIndex(id)),
    STORE_GLOBAL(id: GlobalIndex<u16>) = 0x14 => (rt) => rt.store_global(GlobalIndex(id)),
    LOAD = 0x31 => (rt) => rt.load(),
    STORE = 0x32 => (rt) => rt.store(),
    BOX_ALLOC(id: TypeId<u32>) = 0x40 => (rt) => rt.box_alloc(TypeId(id)),
    ADD = 0x50 => (rt) => rt.add(),
    SUB = 0x51 => (rt) => rt.sub(),
    MUL = 0x52 => (rt) => rt.mul(),
    DIV = 0x53 => (rt) => rt.div(),
    NEG = 0x54 => (rt) => rt.neg(),
    INC = 0x55 => (rt) => rt.inc(),
    MOD = 0x56 => (rt) => rt.modulo(),
    JMP(ca: CodeAddress<u64>) = 0x60 => (rt) => rt.bytecode.jump_to(CodeAddress(ca).0 as usize),
    JMP_IF_TRUE(ca: CodeAddress<u64>) = 0x61 => (rt) => rt.jmp_condition(CodeAddress(ca), true),
    JMP_IF_FALSE(ca: CodeAddress<u64>) = 0x62 => (rt) => rt.jmp_condition(CodeAddress(ca), false),
    CALL(fi: FunctionIndex<u32>) = 0x70 => (rt) => rt.call(FunctionIndex(fi)),
    RET = 0x71 => (rt) => rt.ret(),
    EQ = 0x80 => (rt) => rt.eq(),
    NEQ = 0x81 => (rt) => rt.ne(),
    LT = 0x82 => (rt) => rt.lt(),
    LTE = 0x83 => (rt) => rt.le(),
    GT = 0x84 => (rt) => rt.gt(),
    GTE = 0x85 => (rt) => rt.ge(),
    AND = 0x86 => (rt) => rt.and(),
    OR = 0x87 => (rt) => rt.or(),
    NOT = 0x88 => (rt) => rt.not(),
    POP = 0x90 => (rt) => { rt.pop() },
    DUP = 0x91 => (rt) => rt.dup(),
    HALT = 0xFF => (rt) => rt.halt()
);
