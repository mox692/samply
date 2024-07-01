use criterion::{black_box, criterion_group, criterion_main, Criterion};
use backtrace::Backtrace;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;
#[cfg(target_arch = "x86_64")]
use framehop::x86_64::UnwindRegsX86_64;
#[cfg(target_arch = "x86_64")]
use framehop::x86_64::{CacheX86_64, UnwinderX86_64};
use framehop::Unwinder;
use std::arch::asm;
use std::path::Path;
use std::cell::RefCell;
use std::sync::Mutex;
use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

// unwind_and_resolve
fn full_backtrace(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    c.bench_function("full_backtrace", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = black_box(Backtrace::new());
                black_box(res);
            })
        })
    });
}

// unwind_and_resolve
fn full_framehop(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();


    c.bench_function("full_framehop", |b| {
        rt.block_on(async {
            let mut cache = CacheX86_64::<_>::new();
            let unwinder: UnwinderX86_64<Vec<u8>> = UnwinderX86_64::new();
            let symbol_map = create_symbol_map().await;
            b.iter(move || {
                let res = black_box(backtrace(0, &symbol_map, &unwinder, &mut cache));
                black_box(res);
            });
        });
    });
}

pub async fn backtrace(aslr_offset: u64, symbol_map: &SymbolMap, unwinder: &UnwinderX86_64<Vec<u8>>, cache: &mut CacheX86_64) {

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

    let mut iter = unwinder.iter_frames(rip, regs, cache, &mut read_stack);

    // print frame
    while let Ok(Some(frame)) = iter.next() {
        let addr = frame.address_for_lookup();
        let relative_addr = addr - aslr_offset;
        let symbol = symbol_map
            .lookup(LookupAddress::Relative(relative_addr as u32))
            .await.map(|sym| {
                sym.symbol.name
            }).unwrap_or("default".to_string());
    }

}

async fn create_symbol_map() -> SymbolMap {
    let config = SymbolManagerConfig::default();
    let symbol_manager = SymbolManager::with_config(config);

    let path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => panic!("boooooooon"),
    };

    let binary_path = Path::new(&path);
    let symbol_map: SymbolMap = symbol_manager
        .load_symbol_map_for_binary_at_path(binary_path, None)
        .await
        .unwrap();

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

criterion_group!(framehop_vs_backtrace, full_backtrace, full_framehop);

criterion_main!(framehop_vs_backtrace);