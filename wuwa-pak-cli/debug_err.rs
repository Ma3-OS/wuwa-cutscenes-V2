use std::path::Path;
use unpak::{Pak, Version};

fn main() {
    let hex = "0x6f80948821CA338739A24D4D9F778BCAC0996B2EF2A73897A789C68AFFC13639";
    let bytes = hex::decode(hex.trim_start_matches("0x")).unwrap();
    let path = Path::new("D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk3-WindowsNoEditor.pak");
    
    match Pak::new(path, Version::V12, Some(bytes.as_slice())) {
        Ok(pak) => println!("Success! {} entries", pak.entries().len()),
        Err(e) => println!("Error for V12: {:?}", e),
    }
}
