use crate::GenError;

pub trait CookieFactory: CookieFactorySerializable {
    fn check_gen_size(&self, buf: &[u8]) -> Result<(), GenError> {
        if let Some(size) = self.gen_size() {
            if buf.len() < size {
                return Err(GenError::BufferTooSmall(size));
            }
        }
        Ok(())
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        self.check_gen_size(buf)?;
        self.do_serialize(buf)
    }

    fn serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        self.check_gen_size(buf)?;
        self.do_serialize_le(buf)
    }
}

pub trait CookieFactorySerializable {
    fn gen_size(&self) -> Option<usize> {
        None
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError>;

    fn do_serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        self.do_serialize(buf)
    }
}

impl<T> CookieFactory for T where T: CookieFactorySerializable {}

mod cfimpl;
