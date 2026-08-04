use unpak::Pak;
use std::fs::File;

fn main() {
    let file = File::open("test.pak").unwrap();
    let pak = Pak::new(file);
}
