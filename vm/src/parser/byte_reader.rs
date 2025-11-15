use crate::{aliases::Result, err::RuntimeError};

trait ReadBE {
    const SIZE: usize;
    unsafe fn read_be(buf: &[u8]) -> Self;
}

pub trait ReadSafe: Sized {
    fn read_safe(reader: &mut ByteReader) -> Result<Self>;
}

#[derive(Debug)]
pub struct ByteReader<'r> {
    bytes: &'r [u8],
    offset: usize,
    size: usize,
}

impl<'reader> ByteReader<'reader> {
    pub fn new(bytes: &'reader [u8], size: usize) -> Self {
        Self {
            bytes,
            offset: 0,
            size,
        }
    }

    pub fn read<T: ReadBE>(&mut self) -> Result<T> {
        let start = self.offset;
        self.offset += T::SIZE;
        if self.offset > self.size {
            return Err(RuntimeError::OutOfBounds);
        }
        Ok(unsafe { T::read_be(self.bytes.get_unchecked(start..start + T::SIZE)) })
    }

    pub fn read_safe<T: ReadSafe>(&mut self) -> Result<T> {
        T::read_safe(self)
    }

    pub fn skip(&mut self, n: usize) {
        self.offset += n
    }

    pub fn position(&self) -> usize {
        self.offset
    }

    pub fn jump_to(&mut self, position: usize) -> Result<()> {
        if position > self.size {
            return Err(RuntimeError::OutOfBounds);
        }
        self.offset = position;
        Ok(())
    }
}

macro_rules! impl_readbe {
    ($($type:ty)+) => {
        $(
            impl ReadBE for $type {
                const SIZE: usize = std::mem::size_of::<$type>();

                unsafe fn read_be(buf: &[u8]) -> Self {
                        let value = (buf.as_ptr() as *const $type).read_unaligned();
                        <$type>::from_be(value)
                }
            }
        )+
    };
}

impl_readbe!(u8 u16 u32 u64 i8 i16 i32 i64 usize);

impl<const N: usize> ReadBE for [u8; N] {
    const SIZE: usize = std::mem::size_of::<[u8; N]>();

    unsafe fn read_be(buf: &[u8]) -> Self {
        *(buf.as_ptr() as *const [u8; N])
    }
}

impl ReadBE for f64 {
    const SIZE: usize = std::mem::size_of::<f64>();
    unsafe fn read_be(buf: &[u8]) -> Self {
        let value = (buf.as_ptr() as *const u64).read_unaligned();
        f64::from_bits(value)
    }
}

impl ReadBE for bool {
    const SIZE: usize = std::mem::size_of::<bool>();
    unsafe fn read_be(buf: &[u8]) -> Self {
        (buf.as_ptr() as *const bool).read_unaligned()
    }
}
