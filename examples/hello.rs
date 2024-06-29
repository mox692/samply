//! macos(x86) で, stack unwindがどのようにできるかテスト
//!
//! * 実行スレッドの情報(レジスタの値とか)を取得

#[cfg(target_arch = "x86_64")]
use framehop::x86_64::UnwindRegsX86_64;
#[cfg(target_arch = "x86_64")]
use framehop::x86_64::{CacheX86_64, UnwinderX86_64};
use framehop::Unwinder;
use std::arch::asm;
use std::path::Path;
use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async { fofsadfklhflkadfsa().await });
}
async fn fofsadfklhflkadfsa() {
    bar().await
}
async fn bar() {
    baz().await
}
async fn baz() {
    let mut cache = CacheX86_64::<_>::new();
    let unwinder: UnwinderX86_64<Vec<u8>> = UnwinderX86_64::new();

    let mut read_stack = |addr| {
        if addr % 8 != 0 {
            // Unaligned address
            return Err(());
        }
        // MEMO: シンプルに addr で渡ってきてるメモリの値を読んでるだけ.
        Ok(read_memory(addr))
    };

    // get value of registers
    let (rip, regs) = {
        let mut rip = 0;
        let mut rsp = 0;
        let mut rbp = 0;
        unsafe { asm!("lea {}, [rip]", out(reg) rip) };
        unsafe { asm!("mov {}, rsp", out(reg) rsp) };
        unsafe { asm!("mov {}, rbp", out(reg) rbp) };
        (rip, UnwindRegsX86_64::new(rip, rsp, rbp))
    };

    println!("rip: {:?}", rip as usize as *const ());
    println!("regs: {:?}", &regs);

    let mut iter = unwinder.iter_frames(rip, regs, &mut cache, &mut read_stack);

    let symbol_map = create_symbol_manager().await;

    // print frame
    while let Ok(Some(frame)) = iter.next() {
        for i in (0..8).into_iter() {
            let addr = frame.clone().address() as usize as *const ();
            let addr2 = frame.address_for_lookup() as usize as *const ();
            let s = symbol_map.lookup(LookupAddress::FileOffset(0x9ec9)).await;
            println!(
                "********** {}  addr: {:?}, addr2: {:?} sym: {:?}",
                i,
                addr,
                addr2,
                s // symbol_map.lookup_sync(LookupAddress::FileOffset(addr as usize as u64 - i as u64)),
                  // symbol_map.lookup_sync(LookupAddress::Svma(addr as usize as u64 - i as u64))
            );
        }
        println!("addr: {:?}", frame.address() as u32 as *const ());
    }
    println!("");
}

async fn create_symbol_manager() -> SymbolMap {
    let config = SymbolManagerConfig::default();
    let symbol_manager = SymbolManager::with_config(config);

    let binary_path = Path::new("/Users/s15255/work/samply/target/debug/examples/hello");
    let symbol_map: SymbolMap = symbol_manager
        .load_symbol_map_for_binary_at_path(binary_path, None)
        .await
        .unwrap();

    for (addr, (idx, sym)) in symbol_map.iter_symbols().enumerate() {
        if sym.contains("fofsadfklhflkadfsa") {
            println!("addr: {:?}, sym: {:?}", addr as *const (), sym);
        }
    }

    symbol_map
}

fn read_memory(address: u64) -> u64 {
    // 指定されたメモリ位置から値を読み込む
    let value: usize;
    unsafe {
        value = *(address as *const usize);
    }
    value as u64
}
