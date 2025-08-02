/// Specifies whether an address is relative to the binary base or absolute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupAddress {
    /// Address relative to the binary's base address
    Relative(u64),
    /// Absolute address
    Absolute(u64),
}

/// Information about a resolved symbol.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// The symbol name
    pub name: String,
    /// The address of the symbol (relative to binary base)
    pub address: u64,
    /// Source location information if available
    pub location: Option<SourceLocation>,
    /// Whether this symbol was inlined
    pub inlined: bool,
}

/// Source file location information.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// The source file path
    pub file: String,
    /// The line number (1-based)
    pub line: u32,
    /// The column number (1-based), if available
    pub column: Option<u32>,
}