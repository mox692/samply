use std::env;
use std::path::Path;
use wholesym_lite::{LookupAddress, SymbolManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <binary_path> <address>", args[0]);
        eprintln!("Example: {} /usr/bin/ls 0x1234", args[0]);
        return Ok(());
    }
    
    let binary_path = Path::new(&args[1]);
    let address = if args[2].starts_with("0x") || args[2].starts_with("0X") {
        u64::from_str_radix(&args[2][2..], 16)?
    } else {
        args[2].parse::<u64>()?
    };
    
    let symbol_manager = SymbolManager::new();
    
    println!("Loading symbols from: {}", binary_path.display());
    let symbol_map = match symbol_manager.load_symbol_map_for_binary_at_path(binary_path).await {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error loading symbol map: {}", e);
            return Ok(());
        }
    };
    
    println!("Looking up address: {:#x}", address);
    if let Some(symbol_info) = symbol_map.lookup(LookupAddress::Relative(address)).await {
        println!("Found symbol:");
        println!("  Name: {}", symbol_info.name);
        println!("  Address: {:#x}", symbol_info.address);
        if let Some(loc) = symbol_info.location {
            println!("  Location: {}:{}", loc.file, loc.line);
            if let Some(col) = loc.column {
                println!("  Column: {}", col);
            }
        }
    } else {
        println!("No symbol found at address {:#x}", address);
    }
    
    Ok(())
}