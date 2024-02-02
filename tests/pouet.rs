use std::iter::repeat;

fn slice1(out: &mut [u8]) -> Result<&mut [u8], u32> {
    let data = &b"pouet"[..];
    if out.len() < data.len() {
        Err(42)
    } else {
        out[..data.len()].copy_from_slice(data);
        Ok(&mut out[data.len()..])
    }
}

fn main() {
    let mut v1 = repeat(0u8).take(4).collect::<Vec<_>>();
    let mut v2 = repeat(0u8).take(10).collect::<Vec<_>>();

    println!("res1: {:?}", slice1(&mut v1[..]));
    println!("v1: {:?}", v1);

    println!("res2: {:?}", slice1(&mut v2[..]));
    println!("v2: {:?}", v2);
}
