use crate::{CookieFactory, CookieFactorySerializable, GenError};

use std::mem;

fn zip_copy(from: &[u8], to: &mut [u8]) -> usize {
    let len = from.len();
    for (from, mut to) in from.iter().zip(to.iter_mut()) {
        *to = *from;
    }
    len
}

macro_rules! cookie_factory_number(
    ($t:tt) => (
        impl CookieFactorySerializable for $t {
            fn gen_size(&self) -> Option<usize> {
                Some(mem::size_of_val(self))
            }

            fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
                let len = zip_copy(&self.to_be_bytes()[..], buf);
                Ok(&mut buf[len..])
            }

            fn do_serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
                let len = zip_copy(&self.to_le_bytes()[..], buf);
                Ok(&mut buf[len..])
            }
        }
    );
);

cookie_factory_number!(u8);
cookie_factory_number!(u16);
cookie_factory_number!(u32);
cookie_factory_number!(u64);
cookie_factory_number!(i8);
cookie_factory_number!(i16);
cookie_factory_number!(i32);
cookie_factory_number!(i64);

impl CookieFactorySerializable for f32 {
    fn gen_size(&self) -> Option<usize> {
        Some(mem::size_of_val(self))
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        unsafe { mem::transmute::<f32, u32>(*self) }.do_serialize(buf)
    }

    fn do_serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        unsafe { mem::transmute::<f32, u32>(*self) }.do_serialize_le(buf)
    }
}

impl CookieFactorySerializable for f64 {
    fn gen_size(&self) -> Option<usize> {
        Some(mem::size_of_val(self))
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        unsafe { mem::transmute::<f64, u64>(*self) }.do_serialize(buf)
    }

    fn do_serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        unsafe { mem::transmute::<f64, u64>(*self) }.do_serialize_le(buf)
    }
}

impl<S: CookieFactory> CookieFactorySerializable for [S] {
    fn gen_size(&self) -> Option<usize> {
        self.iter().fold(Some(0), |acc, el| if let (Some(acc), Some(sz)) = (acc, el.gen_size()) { Some(acc + sz) } else { None } )
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        self.iter().fold(Ok(buf), |acc, el| acc.and_then(|buf| el.serialize(buf)))
    }

    fn do_serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        self.iter().fold(Ok(buf), |acc, el| acc.and_then(|buf| el.serialize_le(buf)))
    }
}

impl<'s, S: CookieFactory> CookieFactorySerializable for &'s [S] {
    fn gen_size(&self) -> Option<usize> {
        (*self).gen_size()
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        (*self).do_serialize(buf)
    }

    fn do_serialize_le<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        (*self).do_serialize_le(buf)
    }
}

impl CookieFactorySerializable for str {
    fn gen_size(&self) -> Option<usize> {
        self.as_bytes().gen_size()
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        self.as_bytes().do_serialize(buf)
    }
}

impl<'s> CookieFactorySerializable for &'s str {
    fn gen_size(&self) -> Option<usize> {
        (*self).gen_size()
    }

    fn do_serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], GenError> {
        (*self).do_serialize(buf)
    }
}
