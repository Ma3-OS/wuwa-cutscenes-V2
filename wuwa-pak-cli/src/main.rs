use std::time::Instant;
use reqwest;
use serde::Deserialize;
use unpak::Pak;
use std::io;

#[derive(Deserialize, Debug)]
struct KeyResponse {
    mainKey: String,
    dynamicKeys: Vec<DynamicKey>,
}

#[derive(Deserialize, Debug)]
struct DynamicKey {
    guid: String,
    key: String,
}

fn hex_to_key_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    let hex_clean = hex_str.trim_start_matches("0x");
    hex::decode(hex_clean).map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Wuthering Waves Pak Tester ===");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: wuwa-pak-cli.exe <path-to-pak>");
        std::process::exit(1);
    }
    
    let path = &args[1];
    println!("Target: {}", path);
    
    println!("\n1. Fetching AES keys from yariko0chka...");
    let resp = reqwest::get("https://yarik0chka.github.io/wuwa-keys/keys.json")
        .await?
        .json::<KeyResponse>()
        .await?;
    
    println!("Loaded main key + {} dynamic keys.", resp.dynamicKeys.len());
    
    let main_key = hex_to_key_bytes(&resp.mainKey)?;
    
    println!("\n2. Reading Pak Footer...");
    let mut f = std::fs::File::open(path)?;
    use std::io::{Seek, SeekFrom, Read};
    let mut footer_data = vec![0u8; 221];
    f.seek(SeekFrom::End(-221))?;
    f.read_exact(&mut footer_data)?;
    let mut cur = std::io::Cursor::new(&footer_data);
    let mut guid = [0u8; 16];
    cur.read_exact(&mut guid)?;
    let mut enc = [0u8; 1];
    cur.read_exact(&mut enc)?;
    let mut magic = [0u8; 4];
    cur.read_exact(&mut magic)?;
    let mut ver = [0u8; 4];
    cur.read_exact(&mut ver)?;
    let version = u32::from_le_bytes(ver);
    
    println!("Version: {}", version);
    print!("GUID: ");
    for b in guid.iter() {
        print!("{:02X}", b);
    }
    println!();
    println!("Encrypted: {}", enc[0] != 0);
    
    println!("\n3. Testing keys...");
    print!("Trying mainKey... ");
    let mut found = false;
    match Pak::new_any(path, Some(&main_key)) {
        Ok(pak) => {
            println!("Success! Main key worked.");
            println!("Entries found: {}", pak.entries().len());
            found = true;
        },
        Err(e) => {
            println!("Failed ({:?})", e);
        }
    }
    
    if !found {
        println!("\nBrute-forcing {} dynamic keys...", resp.dynamicKeys.len());
        let start = Instant::now();
        let mut tested = 0;
        for dyn_key in &resp.dynamicKeys {
            if let Ok(key_bytes) = hex_to_key_bytes(&dyn_key.key) {
                match Pak::new_any(&path, Some(&key_bytes)) {
                    Ok(pak) => {
                        println!("\nSuccess! Found matching dynamic key:");
                        println!("GUID: {}", dyn_key.guid);
                        println!("Key: {}", dyn_key.key);
                        println!("Entries found: {}", pak.entries().len());
                        found = true;
                        break;
                    },
                    Err(_) => {
                    }
                }
                tested += 1;
                if tested % 50 == 0 {
                    print!(".");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
        }
        
        println!("\nFinished brute-forcing {} keys in {:.2?}", tested, start.elapsed());
    }

    if !found {
        println!("Failed to mount pak with any known key.");
        std::process::exit(1);
    }
    Ok(())
}
