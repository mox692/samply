use crate::{Error, LookupAddress, SourceLocation, SymbolInfo};
use addr2line::gimli;
use memmap2::Mmap;
use object::{Object, ObjectSection, ObjectSegment, ObjectSymbol};
use std::fs::File;
use std::path::{Path, PathBuf};

pub(crate) struct SymbolMapInner {
    path: PathBuf,
    _mmap: Mmap,
    data: &'static [u8],
    addr2line_context: Option<addr2line::Context<gimli::EndianSlice<'static, gimli::RunTimeEndian>>>,
    base_address: u64,
}

impl SymbolMapInner {
    pub fn load_from_path(path: &Path) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::FileNotFound {
                path: path.to_path_buf(),
            });
        }

        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Leak the mmap to get 'static lifetime. This is safe because we store
        // the mmap in our struct and drop it when the struct is dropped.
        let mmap_box = Box::new(mmap);
        let mmap_ptr = Box::into_raw(mmap_box);
        let mmap = unsafe { &*mmap_ptr };
        let data: &'static [u8] = unsafe {
            std::slice::from_raw_parts(mmap.as_ptr(), mmap.len())
        };
        
        // Parse the object file
        let object = object::File::parse(data)?;
        
        // Determine base address
        let base_address = object
            .segments()
            .filter_map(|seg| {
                if seg.size() > 0 {
                    Some(seg.address())
                } else {
                    None
                }
            })
            .min()
            .unwrap_or(0);
        
        // Try to create addr2line context for DWARF debug info
        let addr2line_context = Self::create_addr2line_context(&object, data)?;
        
        // Recreate the mmap from the raw pointer for storage
        let mmap = unsafe { Box::from_raw(mmap_ptr) };
        
        Ok(Self {
            path: path.to_path_buf(),
            _mmap: *mmap,
            data,
            addr2line_context,
            base_address,
        })
    }
    
    fn create_addr2line_context(
        object: &object::File,
        data: &'static [u8],
    ) -> Result<Option<addr2line::Context<gimli::EndianSlice<'static, gimli::RunTimeEndian>>>, Error> {
        let endian = if object.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };
        
        // Create a closure that loads sections from the static data
        let load_section = |id: gimli::SectionId| -> Result<gimli::EndianSlice<'static, gimli::RunTimeEndian>, gimli::Error> {
            let name = id.name();
            let offset = object
                .section_by_name(name)
                .and_then(|section| {
                    let (offset, size) = section.file_range()?;
                    Some((offset as usize, size as usize))
                })
                .unwrap_or((0, 0));
            
            let section_data = if offset.1 > 0 && offset.0 + offset.1 <= data.len() {
                &data[offset.0..offset.0 + offset.1]
            } else {
                &[]
            };
            
            Ok(gimli::EndianSlice::new(section_data, endian))
        };
        
        let dwarf = gimli::Dwarf::load(&load_section)?;
        
        // Check if we have debug info
        let mut units = dwarf.units();
        if units.next()?.is_none() {
            return Ok(None);
        }
        
        let context = addr2line::Context::from_dwarf(dwarf)?;
        Ok(Some(context))
    }
    
    pub fn lookup(&mut self, address: LookupAddress) -> Option<SymbolInfo> {
        let addr = match address {
            LookupAddress::Relative(offset) => offset,
            LookupAddress::Absolute(abs_addr) => {
                if abs_addr < self.base_address {
                    return None;
                }
                abs_addr - self.base_address
            }
        };
        
        // First try debug info lookup
        if let Some(ref mut context) = self.addr2line_context {
            if let Ok(mut frames) = context.find_frames(addr).skip_all_loads() {
                if let Ok(Some(frame)) = frames.next() {
                    let name = frame
                        .function
                        .as_ref()
                        .and_then(|f| f.demangle().ok())
                        .map(|name| name.into_owned())
                        .unwrap_or_else(|| "<unknown>".to_string());
                    
                    let location = frame.location.as_ref().map(|loc| {
                        SourceLocation {
                            file: loc.file.unwrap_or("<unknown>").to_string(),
                            line: loc.line.unwrap_or(0),
                            column: loc.column,
                        }
                    });
                    
                    return Some(SymbolInfo {
                        name,
                        address: addr,
                        location,
                        inlined: false,
                    });
                }
            }
        }
        
        // Fall back to symbol table lookup
        self.lookup_symbol_table(addr)
    }
    
    fn lookup_symbol_table(&self, address: u64) -> Option<SymbolInfo> {
        // Re-parse the object for symbol table lookup
        let object = object::File::parse(self.data).ok()?;
        
        let mut best_symbol = None;
        let mut best_distance = u64::MAX;
        
        for symbol in object.symbols() {
            if let Ok(name) = symbol.name() {
                if name.is_empty() {
                    continue;
                }
                
                let sym_addr = symbol.address();
                if sym_addr <= address {
                    let distance = address - sym_addr;
                    if distance < best_distance {
                        best_distance = distance;
                        best_symbol = Some((name, sym_addr));
                    }
                }
            }
        }
        
        best_symbol.map(|(name, sym_addr)| {
            // Try to demangle the symbol name
            let demangled_name = if name.starts_with("_Z") || name.starts_with("__Z") {
                // C++ mangled name
                cpp_demangle::Symbol::new(name)
                    .ok()
                    .and_then(|sym| sym.demangle(&cpp_demangle::DemangleOptions::default()).ok())
                    .unwrap_or_else(|| name.to_string())
            } else {
                name.to_string()
            };
            
            SymbolInfo {
                name: demangled_name,
                address: sym_addr,
                location: None,
                inlined: false,
            }
        })
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// Note: addr2line::Context is not thread-safe due to interior mutability.
// We wrap SymbolMapInner in a Mutex in the public API for thread safety.