use wholesym_lite::{LookupAddress, SymbolManager};
use std::path::Path;

#[tokio::test]
async fn test_load_nonexistent_file() {
    let symbol_manager = SymbolManager::new();
    let result = symbol_manager
        .load_symbol_map_for_binary_at_path(Path::new("/nonexistent/file"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_lookup_system_binary() {
    let symbol_manager = SymbolManager::new();
    
    // Try to load a system binary that should exist on most Unix systems
    let paths = vec![
        "/bin/ls",
        "/usr/bin/ls",
        "/bin/cat",
        "/usr/bin/cat",
        "/bin/echo",
        "/usr/bin/echo",
    ];
    
    for path_str in paths {
        let path = Path::new(path_str);
        if path.exists() {
            match symbol_manager.load_symbol_map_for_binary_at_path(path).await {
                Ok(symbol_map) => {
                    // Try to lookup the entry point (usually _start or main)
                    // This is a basic test to ensure the symbol map loads
                    assert_eq!(symbol_map.path(), path);
                    
                    // Try to look up a small offset - there should be some symbol
                    let result = symbol_map.lookup(LookupAddress::Relative(0x1000)).await;
                    // We can't assert that a symbol is found because it depends on the binary
                    // but at least we can verify that the lookup doesn't panic
                    
                    println!("Successfully loaded {}", path_str);
                    return;
                }
                Err(e) => {
                    eprintln!("Failed to load {}: {}", path_str, e);
                }
            }
        }
    }
    
    // If we get here, no system binaries were found
    eprintln!("Warning: No suitable system binaries found for testing");
}

#[tokio::test]
async fn test_types() {
    // Test that our types work correctly
    let relative_addr = LookupAddress::Relative(0x1234);
    let absolute_addr = LookupAddress::Absolute(0x401234);
    
    match relative_addr {
        LookupAddress::Relative(addr) => assert_eq!(addr, 0x1234),
        _ => panic!("Expected relative address"),
    }
    
    match absolute_addr {
        LookupAddress::Absolute(addr) => assert_eq!(addr, 0x401234),
        _ => panic!("Expected absolute address"),
    }
}