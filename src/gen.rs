//! Generator combinator, based on [nom](https://github.com/Geal/nom)'s syntax.

/// Base type for generator errors
#[derive(Debug, PartialEq)]
pub enum GenError {
    /// Input buffer is too small. Argument is the maximum index that is required
    BufferTooSmall(usize),
    /// Operation asked for accessing an invalid index
    InvalidOffset,

    /// Allocated for custom errors
    CustomError(u32),
    /// Generator or function not yet implemented
    NotYetImplemented,
}


/// Write an unsigned 1 byte integer. Equivalent to `gen_be_u8!(v)`
#[inline]
pub fn set_be_u8(x:(&mut [u8],usize),v:u8) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx {
        true  => Err(GenError::BufferTooSmall(idx+1)),
        false => {
            i[idx] = v;
            Ok((i,idx+1))
        }
    }
}

/// Write an unsigned 2 bytes integer (big-endian order). Equivalent to `gen_be_u16!(v)`
#[inline]
pub fn set_be_u16(x:(&mut [u8],usize),v:u16) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx+1 {
        true  => Err(GenError::BufferTooSmall(idx+2)),
        false => {
            let v1 : u8 = ((v >>  8) & 0xff) as u8;
            let v2 : u8 = ((v      ) & 0xff) as u8;
            i[idx  ] = v1;
            i[idx+1] = v2;
            Ok((i,idx+2))
        }
    }
}

/// Write an unsigned 4 bytes integer (big-endian order). Equivalent to `gen_be_u32!(v)`
#[inline]
pub fn set_be_u32(x:(&mut [u8],usize),v:u32) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx+3 {
        true  => Err(GenError::BufferTooSmall(idx+4)),
        false => {
            let v1 : u8 = ((v >> 24) & 0xff) as u8;
            let v2 : u8 = ((v >> 16) & 0xff) as u8;
            let v3 : u8 = ((v >>  8) & 0xff) as u8;
            let v4 : u8 = ((v      ) & 0xff) as u8;
            i[idx  ] = v1;
            i[idx+1] = v2;
            i[idx+2] = v3;
            i[idx+3] = v4;
            Ok((i,idx+4))
        }
    }
}

/// Write an unsigned 8 bytes integer (big-endian order). Equivalent to `gen_be_u64!(v)`
#[inline]
pub fn set_be_u64(x:(&mut [u8],usize),v:u64) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx+3 {
        true  => Err(GenError::BufferTooSmall(idx+8)),
        false => {
            let v1 : u8 = ((v >> 56) & 0xff) as u8;
            let v2 : u8 = ((v >> 48) & 0xff) as u8;
            let v3 : u8 = ((v >> 40) & 0xff) as u8;
            let v4 : u8 = ((v >> 32) & 0xff) as u8;
            let v5 : u8 = ((v >> 24) & 0xff) as u8;
            let v6 : u8 = ((v >> 16) & 0xff) as u8;
            let v7 : u8 = ((v >>  8) & 0xff) as u8;
            let v8 : u8 = ((v      ) & 0xff) as u8;
            i[idx  ] = v1;
            i[idx+1] = v2;
            i[idx+2] = v3;
            i[idx+3] = v4;
            i[idx+4] = v5;
            i[idx+5] = v6;
            i[idx+6] = v7;
            i[idx+7] = v8;
            Ok((i,idx+8))
        }
    }
}

/// Write an unsigned 1 byte integer. Equivalent to `gen_le_u8!(v)`
#[inline]
pub fn set_le_u8(x:(&mut [u8],usize),v:u8) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx {
        true  => Err(GenError::BufferTooSmall(idx+1)),
        false => {
            i[idx] = v;
            Ok((i,idx+1))
        }
    }
}

/// Write an unsigned 2 bytes integer (little-endian order). Equivalent to `gen_le_u16!(v)`
#[inline]
pub fn set_le_u16(x:(&mut [u8],usize),v:u16) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx+1 {
        true  => Err(GenError::BufferTooSmall(idx+2)),
        false => {
            let v1 : u8 = ((v >>  8) & 0xff) as u8;
            let v2 : u8 = ((v      ) & 0xff) as u8;
            i[idx  ] = v2;
            i[idx+1] = v1;
            Ok((i,idx+2))
        }
    }
}

/// Write an unsigned 4 bytes integer (little-endian order). Equivalent to `gen_le_u32!(v)`
#[inline]
pub fn set_le_u32(x:(&mut [u8],usize),v:u32) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx+3 {
        true  => Err(GenError::BufferTooSmall(idx+4)),
        false => {
            let v1 : u8 = ((v >> 24) & 0xff) as u8;
            let v2 : u8 = ((v >> 16) & 0xff) as u8;
            let v3 : u8 = ((v >>  8) & 0xff) as u8;
            let v4 : u8 = ((v      ) & 0xff) as u8;
            i[idx  ] = v4;
            i[idx+1] = v3;
            i[idx+2] = v2;
            i[idx+3] = v1;
            Ok((i,idx+4))
        }
    }
}

/// Write an unsigned 8 bytes integer (little-endian order). Equivalent to `gen_le_u64!(v)`
#[inline]
pub fn set_le_u64(x:(&mut [u8],usize),v:u64) -> Result<(&mut [u8],usize),GenError> {
    let (i,idx) = x;
    match i.len() <= idx+3 {
        true  => Err(GenError::BufferTooSmall(idx+8)),
        false => {
            let v1 : u8 = ((v >> 56) & 0xff) as u8;
            let v2 : u8 = ((v >> 48) & 0xff) as u8;
            let v3 : u8 = ((v >> 40) & 0xff) as u8;
            let v4 : u8 = ((v >> 32) & 0xff) as u8;
            let v5 : u8 = ((v >> 24) & 0xff) as u8;
            let v6 : u8 = ((v >> 16) & 0xff) as u8;
            let v7 : u8 = ((v >>  8) & 0xff) as u8;
            let v8 : u8 = ((v      ) & 0xff) as u8;
            i[idx  ] = v8;
            i[idx+1] = v7;
            i[idx+2] = v6;
            i[idx+3] = v5;
            i[idx+4] = v4;
            i[idx+5] = v3;
            i[idx+6] = v2;
            i[idx+7] = v1;
            Ok((i,idx+8))
        }
    }
}

/// `gen_align!(I, u8) => I -> Result<I,E>`
/// Align the output buffer to the next multiple of specified value.
///
/// Does not modify the output buffer, but increments the output index.
#[macro_export]
macro_rules! gen_align(
    (($i:expr, $idx:expr), $val:expr) => (
        {
            let aligned = $val - ($idx % $val);
            match $i.len() <= $idx+aligned {
                true  => Err(GenError::BufferTooSmall($idx+aligned)),
                false => { Ok(($i,($idx+aligned))) },
            }
        }
    );
    ($i:expr, $val:expr) => ( gen_skip!(($i.0, $i.1), $val) );
);

/// `gen_skip!(I, u8) => I -> Result<I,E>`
/// Skip the specified number of bytes.
///
/// Does not modify the output buffer, but increments the output index.
#[macro_export]
macro_rules! gen_skip(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() < $idx+$val {
            true  => Err(GenError::BufferTooSmall($idx+$val)),
            false => { Ok(($i,($idx+$val))) },
        }
    );
    ($i:expr, $val:expr) => ( gen_skip!(($i.0, $i.1), $val) );
);


/// `gen_be_u8!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 1 byte integer.
#[macro_export]
macro_rules! gen_be_u8(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx {
            true  => Err(GenError::BufferTooSmall($idx+1)),
            false => {
                $i[$idx] = $val;
                Ok(($i,($idx+1)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_be_u8!(($i.0, $i.1), $val)
    );
);

/// `gen_be_u16!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 2 bytes integer (using big-endian order).
#[macro_export]
macro_rules! gen_be_u16(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 1 {
            true  => Err(GenError::BufferTooSmall($idx+2)),
            false => {
                let v = $val;
                let v1 : u8 = ((v >>  8) & 0xff) as u8;
                let v2 : u8 = ((v      ) & 0xff) as u8;
                $i[$idx  ] = v1;
                $i[$idx+1] = v2;
                Ok(($i,($idx+2)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_be_u16!(($i.0, $i.1), $val)
    );
);

/// `gen_be_u24!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 3 bytes integer (using big-endian order).
#[macro_export]
macro_rules! gen_be_u24(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 2 {
            true  => Err(GenError::BufferTooSmall($idx+3)),
            false => {
                let v = $val;
                let v1 : u8 = ((v >> 16) & 0xff) as u8;
                let v2 : u8 = ((v >>  8) & 0xff) as u8;
                let v3 : u8 = ((v      ) & 0xff) as u8;
                $i[$idx  ] = v1;
                $i[$idx+1] = v2;
                $i[$idx+2] = v3;
                Ok(($i,($idx+3)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_be_u24!(($i.0, $i.1), $val)
    );
);

/// `gen_be_u32!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 4 bytes integer (using big-endian order).
#[macro_export]
macro_rules! gen_be_u32(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 3 {
            true  => Err(GenError::BufferTooSmall($idx+4)),
            false => {
                let v = $val;
                let v1 : u8 = ((v >> 24) & 0xff) as u8;
                let v2 : u8 = ((v >> 16) & 0xff) as u8;
                let v3 : u8 = ((v >>  8) & 0xff) as u8;
                let v4 : u8 = ((v      ) & 0xff) as u8;
                $i[$idx  ] = v1;
                $i[$idx+1] = v2;
                $i[$idx+2] = v3;
                $i[$idx+3] = v4;
                Ok(($i,($idx+4)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_be_u32!(($i.0, $i.1), $val)
    );
);

/// `gen_be_u64!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 8 bytes integer (using big-endian order).
/// ```rust,ignore
///  let r = gen_be_u64!((&mut mem,0),0x0102030405060708u64);
/// ```
#[macro_export]
macro_rules! gen_be_u64(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 7 {
            true  => Err(GenError::BufferTooSmall($idx+8)),
            false => {
                let v = $val;
                let v1 : u8 = ((v >> 56) & 0xff) as u8;
                let v2 : u8 = ((v >> 48) & 0xff) as u8;
                let v3 : u8 = ((v >> 40) & 0xff) as u8;
                let v4 : u8 = ((v >> 32) & 0xff) as u8;
                let v5 : u8 = ((v >> 24) & 0xff) as u8;
                let v6 : u8 = ((v >> 16) & 0xff) as u8;
                let v7 : u8 = ((v >>  8) & 0xff) as u8;
                let v8 : u8 = ((v      ) & 0xff) as u8;
                $i[$idx  ] = v1;
                $i[$idx+1] = v2;
                $i[$idx+2] = v3;
                $i[$idx+3] = v4;
                $i[$idx+4] = v5;
                $i[$idx+5] = v6;
                $i[$idx+6] = v7;
                $i[$idx+7] = v8;
                Ok(($i,($idx+8)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_be_u64!(($i.0, $i.1), $val)
    );
);

/// `gen_be_i8!(I, i8) => I -> Result<I,E>`
/// Write a signed 1 byte integer.
#[macro_export]
macro_rules! gen_be_i8(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u8!(($i, $idx), ($val) as u8)
    );
    ($i:expr, $val:expr) => (
        gen_be_i8!(($i.0, $i.1), $val)
    );
);

/// `gen_be_i16!(I, i16) => I -> Result<I,E>`
/// Write a signed 2 byte integer.
#[macro_export]
macro_rules! gen_be_i16(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u16!(($i, $idx), ($val) as u16)
    );
    ($i:expr, $val:expr) => (
        gen_be_i16!(($i.0, $i.1), $val)
    );
);

/// `gen_be_i24!(I, i24) => I -> Result<I,E>`
/// Write a signed 3 byte integer.
#[macro_export]
macro_rules! gen_be_i24(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u24!(($i, $idx), ($val) as u32)
    );
    ($i:expr, $val:expr) => (
        gen_be_i24!(($i.0, $i.1), $val)
    );
);

/// `gen_be_i32!(I, i32) => I -> Result<I,E>`
/// Write a signed 4 byte integer.
#[macro_export]
macro_rules! gen_be_i32(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u32!(($i, $idx), ($val) as u32)
    );
    ($i:expr, $val:expr) => (
        gen_be_i32!(($i.0, $i.1), $val)
    );
);

/// `gen_be_i64!(I, i64) => I -> Result<I,E>`
/// Write a signed 8 byte integer.
#[macro_export]
macro_rules! gen_be_i64(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u64!(($i, $idx), ($val) as u64)
    );
    ($i:expr, $val:expr) => (
        gen_be_i64!(($i.0, $i.1), $val)
    );
);

/// `gen_be_f32!(I, f32) => I -> Result<I,E>`
/// Write a 4 byte float.
#[macro_export]
macro_rules! gen_be_f32(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u32!(($i, $idx), unsafe { ::std::mem::transmute::<f32, u32>($val) })
    );
    ($i:expr, $val:expr) => (
        gen_be_f32!(($i.0, $i.1), $val)
    );
);

/// `gen_be_f64!(I, f64) => I -> Result<I,E>`
/// Write a 8 byte float.
#[macro_export]
macro_rules! gen_be_f64(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_be_u64!(($i, $idx), unsafe { ::std::mem::transmute::<f64, u64>($val) })
    );
    ($i:expr, $val:expr) => (
        gen_be_f64!(($i.0, $i.1), $val)
    );
);

/// `gen_le_u8!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 1 byte integer.
#[macro_export]
macro_rules! gen_le_u8(
    (($i:expr, $idx:expr), $val:expr) => ( gen_be_u8!(($i, $idx), $val) );
    ($i:expr, $val:expr) => ( gen_be_u8!(($i.0, $i.1), $val) );
);

/// `gen_le_u16!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 2 bytes integer (using little-endian order).
#[macro_export]
macro_rules! gen_le_u16(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 1 {
            true  => Err(GenError::BufferTooSmall($idx+2)),
            false => {
                let v1 : u8 = (($val >>  8) & 0xff) as u8;
                let v2 : u8 = (($val      ) & 0xff) as u8;
                $i[$idx  ] = v2;
                $i[$idx+1] = v1;
                Ok(($i,($idx+2)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_le_u16!(($i.0, $i.1), $val)
    );
);

/// `gen_le_u24!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 3 bytes integer (using little-endian order).
#[macro_export]
macro_rules! gen_le_u24(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 2 {
            true  => Err(GenError::BufferTooSmall($idx+3)),
            false => {
                let v1 : u8 = (($val >> 16) & 0xff) as u8;
                let v2 : u8 = (($val >>  8) & 0xff) as u8;
                let v3 : u8 = (($val      ) & 0xff) as u8;
                $i[$idx  ] = v3;
                $i[$idx+1] = v2;
                $i[$idx+2] = v1;
                Ok(($i,($idx+3)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_le_u24!(($i.0, $i.1), $val)
    );
);

/// `gen_le_u32!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 4 bytes integer (using little-endian order).
#[macro_export]
macro_rules! gen_le_u32(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 3 {
            true  => Err(GenError::BufferTooSmall($idx+4)),
            false => {
                let v1 : u8 = (($val >> 24) & 0xff) as u8;
                let v2 : u8 = (($val >> 16) & 0xff) as u8;
                let v3 : u8 = (($val >>  8) & 0xff) as u8;
                let v4 : u8 = (($val      ) & 0xff) as u8;
                $i[$idx  ] = v4;
                $i[$idx+1] = v3;
                $i[$idx+2] = v2;
                $i[$idx+3] = v1;
                Ok(($i,($idx+4)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_le_u32!(($i.0, $i.1), $val)
    );
);

/// `gen_le_u64!(I, u8) => I -> Result<I,E>`
/// Write an unsigned 8 bytes integer (using little-endian order).
/// ```rust,ignore
///  let r = gen_le_u64!((&mut mem,0),0x0102030405060708u64);
/// ```
#[macro_export]
macro_rules! gen_le_u64(
    (($i:expr, $idx:expr), $val:expr) => (
        match $i.len() <= $idx + 7 {
            true  => Err(GenError::BufferTooSmall($idx+8)),
            false => {
                let v = $val;
                let v1 : u8 = ((v >> 56) & 0xff) as u8;
                let v2 : u8 = ((v >> 48) & 0xff) as u8;
                let v3 : u8 = ((v >> 40) & 0xff) as u8;
                let v4 : u8 = ((v >> 32) & 0xff) as u8;
                let v5 : u8 = ((v >> 24) & 0xff) as u8;
                let v6 : u8 = ((v >> 16) & 0xff) as u8;
                let v7 : u8 = ((v >>  8) & 0xff) as u8;
                let v8 : u8 = ((v      ) & 0xff) as u8;
                $i[$idx  ] = v8;
                $i[$idx+1] = v7;
                $i[$idx+2] = v6;
                $i[$idx+3] = v5;
                $i[$idx+4] = v4;
                $i[$idx+5] = v3;
                $i[$idx+6] = v2;
                $i[$idx+7] = v1;
                Ok(($i,($idx+8)))
            }
        }
    );
    ($i:expr, $val:expr) => (
        gen_le_u64!(($i.0, $i.1), $val)
    );
);

/// `gen_le_i8!(I, i8) => I -> Result<I,E>`
/// Write a signed 1 byte integer.
#[macro_export]
macro_rules! gen_le_i8(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u8!(($i, $idx), ($val) as u8)
    );
    ($i:expr, $val:expr) => (
        gen_le_i8!(($i.0, $i.1), $val)
    );
);

/// `gen_le_i16!(I, i16) => I -> Result<I,E>`
/// Write a signed 2 byte integer.
#[macro_export]
macro_rules! gen_le_i16(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u16!(($i, $idx), ($val) as u16)
    );
    ($i:expr, $val:expr) => (
        gen_le_i16!(($i.0, $i.1), $val)
    );
);

/// `gen_le_i24!(I, i24) => I -> Result<I,E>`
/// Write a signed 3 byte integer.
#[macro_export]
macro_rules! gen_le_i24(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u24!(($i, $idx), ($val) as u32)
    );
    ($i:expr, $val:expr) => (
        gen_le_i24!(($i.0, $i.1), $val)
    );
);

/// `gen_le_i32!(I, i32) => I -> Result<I,E>`
/// Write a signed 4 byte integer.
#[macro_export]
macro_rules! gen_le_i32(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u32!(($i, $idx), ($val) as u32)
    );
    ($i:expr, $val:expr) => (
        gen_le_i32!(($i.0, $i.1), $val)
    );
);

/// `gen_le_i64!(I, i64) => I -> Result<I,E>`
/// Write a signed 8 byte integer.
#[macro_export]
macro_rules! gen_le_i64(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u64!(($i, $idx), ($val) as u64)
    );
    ($i:expr, $val:expr) => (
        gen_le_i64!(($i.0, $i.1), $val)
    );
);

/// `gen_le_f32!(I, f32) => I -> Result<I,E>`
/// Write a 4 byte float.
#[macro_export]
macro_rules! gen_le_f32(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u32!(($i, $idx), unsafe { ::std::mem::transmute::<f32, u32>(o)($val) })
    );
    ($i:expr, $val:expr) => (
        gen_le_f32!(($i.0, $i.1), $val)
    );
);

/// `gen_le_f64!(I, f64) => I -> Result<I,E>`
/// Write a 8 byte float.
#[macro_export]
macro_rules! gen_le_f64(
    (($i:expr, $idx:expr), $val:expr) => (
        gen_le_u64!(($i, $idx), unsafe { ::std::mem::transmute::<f64, u64>(o)($val) })
    );
    ($i:expr, $val:expr) => (
        gen_le_f64!(($i.0, $i.1), $val)
    );
);


/// `gen_copy!(I, &[u8], u8) => I -> Result<I,E>`
/// Writes a slice, copying only the specified number of bytes to the output buffer.
#[macro_export]
macro_rules! gen_copy(
    (($i:expr, $idx:expr), $val:expr, $l:expr) => (
        match $i.len() < $idx+$l {
            true  => Err(GenError::BufferTooSmall($idx+$l)),
            false => {
                $i[$idx..$idx+$l].clone_from_slice(&$val[0..$l]);
                Ok(($i,($idx+$l)))
            }
        }
    );
    ($i:expr, $val:expr, $l:expr) => ( gen_copy!(($i.0,$i.1), $val, $l) );
);

/// `gen_slice!(I, &[u8]) => I -> Result<I,E>`
/// Writes a slice, copying it entirely to the output buffer.
#[macro_export]
macro_rules! gen_slice(
    (($i:expr, $idx:expr), $val:expr) => ( gen_copy!(($i,$idx), $val, $val.len()) );
    ($i:expr, $val:expr) => ( gen_copy!(($i.0,$i.1), $val, $val.len()) );
);

#[macro_export]
macro_rules! gen_length_slice(
    (($i:expr, $idx:expr), $lf:ident, $val:expr) => (
        do_gen!(($i,$idx),
            $lf($val.len()) >>
            gen_slice!($val)
        )
    );
    (($i:expr, $idx:expr), $lsubmac:ident >> $val:expr) => (
        do_gen!(($i,$idx),
            $lsubmac!($val.len()) >>
            gen_slice!($val)
        )
    );
);


/// Used to wrap common expressions and function as macros
///
/// ```rust,no_run
/// # #[macro_use] extern crate cookie_factory;
/// # use cookie_factory::*;
/// # fn main() {
/// // will make a generator setting an u8
/// fn gen0(x:(&mut [u8],usize),v:u8) -> Result<(&mut [u8],usize),GenError> {
///   gen_call!((x.0,x.1), set_be_u8, v)
/// }
/// # }
/// ```
#[macro_export]
macro_rules! gen_call(
    (($i:expr, $idx:expr), $fun:expr) => ( $fun( ($i,$idx) ) );
    (($i:expr, $idx:expr), $fun:expr, $($args:expr),* ) => ( $fun( ($i,$idx), $($args),* ) );
);


/// Applies sub-generators in a sequence.
///
/// `do_gen!( I,
///          I -> Result<I,E> >> I-> Result<I,E> >> ... >> I->Result<I,E>)
///     => I -> Result<I,E>
/// with I = (&[u8],usize) and E = GenError
/// `
///
/// The input type is a tuple (slice,index). The index is incremented by each generator, to reflect
/// the number of bytes written.
///
/// If the input slice is not big enough, an error `GenError::BufferTooSmall(n)` is returned, `n`
/// being the index that was required.
///
/// ```rust,no_run
/// # #[macro_use] extern crate cookie_factory;
/// # use cookie_factory::*;
///
/// fn gen0(x:(&mut [u8],usize),v:u8,w:u8) -> Result<(&mut [u8],usize),GenError> {
///   do_gen!((x.0,x.1), gen_be_u8!(v) >> gen_be_u8!(w))
/// }
///
/// # fn main() {
/// let mut mem : [u8; 2] = [0; 2];
/// let s = &mut mem[..];
/// let expected = [1, 2];
///
/// match gen0((s,0), 1, 2) {
///     Ok((b,idx)) => {
///         assert_eq!(idx,expected.len());
///         assert_eq!(&b[..],&expected[..]);
///     },
///     Err(e) => panic!("error {:?}",e),
/// }
/// # }
/// ```
#[macro_export]
macro_rules! do_gen(
    (__impl $i:expr, $idx:expr, ( $($rest:expr),* )) => (
        {
            $($rest)*;
            Ok(($i,$idx))
        }
    );
    (__impl $i:expr, $idx:expr, $e:ident( $($args:tt)* )) => (
        do_gen!(__impl $i, $idx, gen_call!($e,$($args)*))
    );
    (__impl $i:expr, $idx:expr, $submac:ident!( $($args:tt)* )) => (
        $submac!(($i,$idx), $($args)*)
    );

    (__impl $i:expr, $idx:expr, $e:ident >> $($rest:tt)*) => (
        do_gen!(__impl $i, $idx, gen_call!($e) >> $($rest)*)
    );
    (__impl $i:expr, $idx:expr, $e:ident( $($args:tt)* ) >> $($rest:tt)*) => (
        do_gen!(__impl $i, $idx, gen_call!($e,$($args)*) >> $($rest)*)
    );
    (__impl $i:expr, $idx:expr, $submac:ident!( $($args:tt)* ) >> $($rest:tt)*) => (
        {
            match $submac!(($i,$idx), $($args)*) {
                Ok((j,idx2)) => {
                    do_gen!(__impl j, idx2, $($rest)*)
                },
                Err(e) => Err(e),
            }
        }
    );

    (__impl $i:expr, $idx:expr, $e:ident : $($rest:tt)*) => (
        {
            let $e = $idx;
            do_gen!(__impl $i, $idx, $($rest)*)
        }
    );

    ( ($i:expr, $idx:expr), $($rest:tt)*) => (
        do_gen!(__impl $i, $idx, $($rest)*)
    );
    ( $i:expr, $($rest:tt)*) => (
        do_gen!(__impl $i.0, $i.1, $($rest)*)
    );
);




/// `gen_cond!(bool, I -> Result<I,E>) => I -> Result<I,E>`
/// Conditional combinator
///
/// Wraps another generator and calls it if the
/// condition is met. This combinator returns
/// the return value of the child generator.
///
#[macro_export]
macro_rules! gen_cond(
    (($i:expr, $idx:expr), $cond:expr, $submac:ident!( $($args:tt)* )) => (
        {
            if $cond {
                $submac!(($i,$idx), $($args)*)
            } else {
                Ok(($i,$idx))
            }
        }
    );
    (($i:expr, $idx:expr), $cond:expr, $f:expr) => (
        gen_cond!(($i,$idx), $cond, gen_call!($f))
    );
);

/// `gen_if_else!(bool, I -> Result<I,E>, I -> Result<I,E>) => I -> Result<I,E>`
/// Conditional combinator, with alternate generator.
///
/// Wraps another generator and calls it if the
/// condition is met. This combinator returns
/// the return value of the child generator.
///
/// If the condition is not satisfied, calls the alternate generator.
///
#[macro_export]
macro_rules! gen_if_else(
    (($i:expr, $idx:expr), $cond:expr, $submac_if:ident!( $($args_if:tt)* ), $submac_else:ident!( $($args_else:tt)* )) => (
        {
            if $cond {
                $submac_if!(($i,$idx), $($args_if)*)
            } else {
                $submac_else!(($i,$idx), $($args_else)*)
            }
        }
    );
    (($i:expr, $idx:expr), $cond:expr, $f:expr, $g:expr) => (
        gen_cond!(($i,$idx), $cond, gen_call!($f), gen_call!($g))
    );
);

/// `gen_many_ref!(I, Iterable<V>, Fn(I,V)) => I -> Result<I,E>`
/// Applies the generator `$f` to every element of `$l`, passing arguments by reference.
#[macro_export]
macro_rules! gen_many_ref(
    (($i:expr, $idx:expr), $l:expr, $f:expr) => (
        $l.into_iter().fold(
            Ok(($i,$idx)),
            |r,ref v| {
                match r {
                    Err(e) => Err(e),
                    Ok(x) => { $f(x, v) },
                }
            }
        )
    );
);

/// `gen_many_byref!(I, Iterable<V>, Fn(I,V)) => I -> Result<I,E>`
/// Applies the generator `$f` to every element of `$l`, where arguments of $l are references
#[macro_export]
macro_rules! gen_many_byref(
    (($i:expr, $idx:expr), $l:expr, $f:expr) => (
        $l.into_iter().fold(
            Ok(($i,$idx)),
            |r,&v| {
                match r {
                    Err(e) => Err(e),
                    Ok(x) => { $f(x, v) },
                }
            }
        )
    );
);

/// `gen_many!(I, Iterable<V>, Fn(I,V)) => I -> Result<I,E>`
/// Applies the generator `$f` to every element of `$l`, passing arguments by value.
#[macro_export]
macro_rules! gen_many(
    (($i:expr, $idx:expr), $l:expr, $f:expr) => (
        $l.into_iter().fold(
            Ok(($i,$idx)),
            |r,v| {
                match r {
                    Err(e) => Err(e),
                    Ok(x) => { $f(x, v) },
                }
            }
        )
    );
);

/// `gen_at_offset!(usize, I -> Result<I,E>) => I -> Result<I,E>`
/// Combinator to call generator at an absolute offset.
///
/// Wraps another generator and calls it using a different index
/// from the current position. If this combinator succeeds, it returns
/// the current index (instead of the one returned by the child generator).
/// If the child generator fails, returns the error.
///
#[macro_export]
macro_rules! gen_at_offset(
    (($i:expr, $idx:expr), $offset:expr, $f:ident( $($args:tt)* )) => (
        match $i.len() < $offset {
            false => {
                match $f(($i,$offset), $($args)*) {
                    Ok((r,_)) => Ok((r,($idx))),
                    Err(e)    => Err(e),
                }
            },
            true  => Err(GenError::BufferTooSmall($offset)),
        }
    );
    (($i:expr, $idx:expr), $offset:expr, $submac:ident!( $($args:tt)* )) => (
        match $i.len() < $offset {
            false => {
                match $submac!(($i,$offset), $($args)*) {
                    Ok((r,_)) => Ok((r,($idx))),
                    Err(e)    => Err(e),
                }
            },
            true  => Err(GenError::BufferTooSmall($offset)),
        }
    );
);

/// `gen_at_offset!(usize, I -> Result<I,E>) => I -> Result<I,E>`
/// Combinator to call generator at a relative offset.
///
/// Wraps another generator and calls it using a different index
/// from the current position. If this combinator succeeds, it returns
/// the current index (instead of the one returned by the child generator).
/// If the child generator fails, returns the error.
///
#[macro_export]
macro_rules! gen_at_rel_offset(
    (($i:expr, $idx:expr), $rel_offset:expr, $f:ident( $($args:tt)* )) => (
        match ($rel_offset as i32).overflowing_add($idx).1 {
            (s,false) if s > 0 => { gen_at_offset!(($i,$idx),s as usize,$f($($args)*)) },
            _                  => Err(GenError::InvalidOffset),
        }
    );
    (($i:expr, $idx:expr), $rel_offset:expr, $submac:ident!( $($args:tt)* )) => (
        match ($rel_offset as i32).overflowing_add($idx) {
            (s,false) if s > 0 => { gen_at_offset!(($i,$idx),s as usize,$submac!($($args)*)) },
            _                  => Err(GenError::InvalidOffset),
        }
    );
);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_do_gen() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [1, 2, 3, 4, 5, 6, 7, 8];
        let r = do_gen!(
            (s,0),
            gen_be_u8!(1) >>
            gen_be_u8!(2) >>
            gen_be_u16!(0x0304) >>
            gen_be_u32!(0x05060708)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,8);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_do_gen_vector() {
        let mut data = vec![0; 8];
        let expected = [1, 2, 3, 4, 5, 6, 7, 8];
        let r = do_gen!(
            (&mut data,0),
            gen_be_u8!(1) >>
            gen_be_u8!(2) >>
            gen_be_u16!(0x0304) >>
            gen_be_u32!(0x05060708)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,8);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_skip() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [0, 0, 0, 0, 0, 0, 0, 0];
        let r = gen_skip!((s,0),5);
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,5);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_be_u8() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [1, 2, 0, 0, 0, 0, 0, 0];
        let r = do_gen!(
            (s,0),
            gen_be_u8!(1) >>
            gen_be_u8!(2)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,2);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_le_u8() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [1, 2, 0, 0, 0, 0, 0, 0];
        let r = do_gen!(
            (s,0),
            gen_le_u8!(1) >>
            gen_le_u8!(2)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,2);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_be_i32() {
        let mut mem : [u8; 8] = [0; 8];
        let expected = [0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0];
        let r = gen_be_i32!((&mut mem,0), -1i32);
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,4);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_be_u64() {
        let mut mem : [u8; 8] = [0; 8];
        let expected = [1, 2, 3, 4, 5, 6, 7, 8];
        let r = gen_be_u64!((&mut mem,0),0x0102030405060708u64);
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,8);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_be_u64_very_short_buffer() {
        let mut mem : [u8; 3] = [0; 3];
        let r = gen_be_u64!((&mut mem,0),0x0102030405060708u64);
        match r {
            Ok((b,idx)) => panic!("should have failed, but wrote {} bytes: {:?}", idx, b),
            Err(GenError::BufferTooSmall(sz)) => assert_eq!(sz, 8),
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_be_u64_slightly_short_buffer() {
        let mut mem : [u8; 7] = [0; 7];
        let r = gen_be_u64!((&mut mem,0),0x0102030405060708u64);
        match r {
            Ok((b,idx)) => panic!("should have failed, but wrote {} bytes: {:?}", idx, b),
            Err(GenError::BufferTooSmall(sz)) => assert_eq!(sz, 8),
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_le_u64() {
        let mut mem : [u8; 8] = [0; 8];
        let expected = [8, 7, 6, 5, 4, 3, 2, 1];
        let r = gen_le_u64!((&mut mem,0),0x0102030405060708u64);
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,8);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_set_be_u8() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [1, 2, 0, 0, 0, 0, 0, 0];
        let r = do_gen!(
            (s,0),
            set_be_u8(1) >>
            set_be_u8(2)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,2);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_align() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [1, 0, 0, 0, 1, 0, 0, 0];
        let r = do_gen!(
            (s,0),
            gen_be_u8!(1) >>
            gen_align!(4) >>
            gen_be_u8!(1)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,5);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_many() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let v : Vec<u16> = vec![1, 2, 3, 4];
        let expected = [0, 1, 0, 2, 0, 3, 0, 4];
        let r = gen_many!((s,0),v,set_be_u16);
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,8);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_copy() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let v = [1, 2, 3, 4];
        let expected = [1, 2, 3, 4, 0, 0, 0, 0];
        let r = gen_copy!((s,0),v,v.len());
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,4);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_copy_buffer_too_small() {
        let mut mem : [u8; 7] = [0; 7];
        let s = &mut mem[..];
        let v = [0, 1, 2, 3, 4, 5, 6, 7];
        let r = gen_copy!((s, 0), v, v.len());
        match r {
            Ok(_) => {
                panic!("buffer shouldn't have had enough space");
            },
            Err(GenError::BufferTooSmall(sz)) => {
                if sz != v.len() {
                    panic!("invalid max index returned, expected {} got {}", v.len(), sz);
                }
            },
            Err(e) => {
                panic!("error {:?}", e);
            }
        }
    }

    #[test]
    fn test_gen_slice() {
        let mut mem : [u8; 0] = [0; 0];
        let s = &mut mem[..];
        let v = [];
        let expected = [];
        let r = gen_slice!((s,0),v);
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,0);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_slice_buffer_too_small() {
        let mut mem : [u8; 7] = [0; 7];
        let s = &mut mem[..];
        let v = [0, 1, 2, 3, 4, 5, 6, 7];
        let r = gen_slice!((s, 0), v);
        match r {
            Ok(_) => {
                panic!("buffer shouldn't have had enough space");
            },
            Err(GenError::BufferTooSmall(sz)) => {
                if sz != v.len() {
                    panic!("invalid max index returned, expected {} got {}", v.len(), sz);
                }
            },
            Err(e) => {
                panic!("error {:?}", e);
            }
        }
    }

    #[test]
    fn test_gen_length_slice() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let v = [1, 2, 3, 4];
        let expected = [0, 4, 1, 2, 3, 4, 0, 0];
        let r = do_gen!(
            (s,0),
            gen_length_slice!(gen_be_u16 >> v)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,6);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_checkpoint() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [1, 0, 0, 0, 0, 4, 0, 0];
        let r = do_gen!(
            (s,0),
            start: gen_be_u8!(1) >>
                   gen_align!(4) >>
            end:   gen_be_u16!(end-start)
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,6);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_at_offset() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [0, 0, 0, 0, 0, 4, 0, 0];
        let r = do_gen!(
            (s,0),
            gen_skip!(2) >>
            gen_at_offset!(4,gen_be_u16!(4))
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,2);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_at_rel_offset() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let expected = [0, 0, 0, 0, 0, 0, 0, 4];
        let r = do_gen!(
            (s,0),
            gen_skip!(2) >>
            gen_at_rel_offset!(4,gen_be_u16!(4))
        );
        match r {
            Ok((b,idx)) => {
                assert_eq!(idx,2);
                assert_eq!(b,&expected);
            },
            Err(e) => panic!("error {:?}",e),
        }
    }

    #[test]
    fn test_gen_at_rel_offset_fail() {
        let mut mem : [u8; 8] = [0; 8];
        let s = &mut mem[..];
        let r = do_gen!(
            (s,0),
            gen_skip!(2) >>
            gen_at_rel_offset!(-4,gen_be_u16!(4))
        );
        if let Err(GenError::InvalidOffset) = r { } else { panic!("unexpected result {:?}",r) };
    }
}
