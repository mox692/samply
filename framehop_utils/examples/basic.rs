/// Basic usage.
use framehop_utils::{read_aslr_offset, UnwindBuilderX86_64, SymbolMapBuilder, SymbolManager, LookupAddress, SymbolManagerConfig, SymbolMap};

#[tokio::main]
async fn main() {
    let symbol_map = SymbolMapBuilder::new().build().await;
    let mut unwinder = UnwindBuilderX86_64::new().build();
    let mut iter = unwinder.unwind();
    let aslr_offset = read_aslr_offset().unwrap();

    while let Some(frame) = iter.next() {
        let symbol = symbol_map.lookup(LookupAddress::Relative((frame.address_for_lookup() - aslr_offset) as u32)).await;
        println!("frame: {:?} symbol: {:?}", &frame, &symbol.map(|s| s.symbol.name));
    }
}
