fn main() {
    let bitfield: u32 = 0x80000000;
    let bit = 31u32;
    println!("result: {}", (bitfield & (1u32 << bit)) != 0);
}
