//! `wholesym-lite` is a minimal symbol resolution library for local binary files.
//!
//! This is a lightweight alternative to the full `wholesym` crate, focused only on
//! resolving symbols from local binary files without any network or symbol server
//! functionality.
//!
//! # Example
//!
//! ```no_run
//! use wholesym_lite::{SymbolManager, LookupAddress};
//! use std::path::Path;
//!
//! # async fn run() -> Result<(), wholesym_lite::Error> {
//! let symbol_manager = SymbolManager::new();
//! let symbol_map = symbol_manager
//!     .load_symbol_map_for_binary_at_path(Path::new("/usr/bin/ls"))
//!     .await?;
//!
//! if let Some(symbol_info) = symbol_map.lookup(LookupAddress::Relative(0x1234)).await {
//!     println!("Symbol: {} at {:#x}", symbol_info.name, symbol_info.address);
//!     if let Some(location) = symbol_info.location {
//!         println!("  at {}:{}", location.file, location.line);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;

pub use addr2line;
pub use object;

mod error;
mod symbol_map;
mod types;

pub use error::Error;
pub use types::{LookupAddress, SourceLocation, SymbolInfo};

/// The main entry point for symbol resolution.
pub struct SymbolManager {
    // For now, we don't need any configuration
}

impl SymbolManager {
    /// Create a new `SymbolManager`.
    pub fn new() -> Self {
        Self {}
    }

    /// Load a symbol map for a binary file at the given path.
    pub async fn load_symbol_map_for_binary_at_path(
        &self,
        path: &Path,
    ) -> Result<Arc<SymbolMap>, Error> {
        let map = symbol_map::SymbolMapInner::load_from_path(path)?;
        Ok(Arc::new(self::SymbolMap(std::sync::Mutex::new(map))))
    }
}

impl Default for SymbolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A loaded symbol map that can be used to resolve addresses to symbols.
pub struct SymbolMap(std::sync::Mutex<symbol_map::SymbolMapInner>);

impl SymbolMap {
    /// Look up symbol information for the given address.
    pub async fn lookup(&self, address: LookupAddress) -> Option<SymbolInfo> {
        self.0.lock().unwrap().lookup(address)
    }

    /// Get the path to the binary file this symbol map was loaded from.
    pub fn path(&self) -> std::path::PathBuf {
        self.0.lock().unwrap().path().to_path_buf()
    }
}
