# wholesym-lite

A minimal symbol resolution library for local binary files.

This is a lightweight alternative to the full `wholesym` crate, focused only on resolving symbols from local binary files without any network or symbol server functionality.

## Features

- Parse ELF, Mach-O, and PE binary formats
- DWARF debug information support
- Symbol table lookups
- Minimal dependencies
- No network functionality

## Usage

```rust
use wholesym_lite::{SymbolManager, LookupAddress};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), wholesym_lite::Error> {
    let symbol_manager = SymbolManager::new();
    let symbol_map = symbol_manager
        .load_symbol_map_for_binary_at_path(Path::new("/usr/bin/ls"))
        .await?;
    
    if let Some(symbol_info) = symbol_map.lookup(LookupAddress::Relative(0x1234)).await {
        println!("Symbol: {} at {:#x}", symbol_info.name, symbol_info.address);
        if let Some(location) = symbol_info.location {
            println!("  at {}:{}", location.file, location.line);
        }
    }
    Ok(())
}
```

## Dependencies

This crate uses only essential dependencies:
- `object` - For parsing binary formats
- `addr2line` - For DWARF debug information
- `memmap2` - For memory-mapped file access
- `cpp_demangle` - For demangling C++ symbols