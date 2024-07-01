//! An example linux program that get a stack unwinding by using framehop
//! and resolve symbols by using wholesym.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;
#[cfg(target_arch = "x86_64")]
use framehop::x86_64::UnwindRegsX86_64;
#[cfg(target_arch = "x86_64")]
use framehop::x86_64::{CacheX86_64, UnwinderX86_64};
use framehop::Unwinder;
use std::arch::asm;
use std::cell::RefCell;
use std::sync::Mutex;
use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

static ASLR_OFFSET: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));

fn read_process_map() -> Result<u64, std::io::Error> {
    let path = "/proc/self/maps";
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut start_address: u64 = 0;

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            let start_address_str = line.split('-').next().unwrap();
            start_address = u64::from_str_radix(start_address_str, 16).unwrap();
            println!("The start address of the first memory range is: {:x}", start_address);
        }
        println!("{}", line);
    }

    Ok(start_address)
}

fn main() {
    let pid = process::id();
    println!("PID is {}", pid);

    match read_process_map() {
        Ok(start_address) => {
            println!("main:          {:?}", main as u64);
            println!("start_address: {:?}", start_address);
            println!("relative main: {:?}",  main as u64 - start_address );

            let guard = ASLR_OFFSET.lock().unwrap();
            guard.replace(start_address);
        },
        Err(e) => eprintln!("Failed to read process map: {}", e),
    }


    run()
}

fn run() {
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

    println!("rip: {:?}", rip); // debug-rip: 0x100047ca2 , binary-rip: 0x10345aca2, 0x109d26ca2, 0x108dd2ca2
    println!("regs: {:?}", &regs);

    let mut iter = unwinder.iter_frames(rip, regs, &mut cache, &mut read_stack);

    let symbol_map = create_symbol_map().await;

    let aslr_offset = ASLR_OFFSET.lock().unwrap().borrow().clone();
    println!("aslr_offset: {:?}", aslr_offset);
    // print frame
    while let Ok(Some(frame)) = iter.next() {
        let addr = frame.address_for_lookup();
        // since we are searching binary file (not dylib).
        let relative_addr = addr - aslr_offset;
        let symbol = symbol_map
            .lookup(LookupAddress::Relative(relative_addr as u32))
            .await.map(|sym| {
                sym.symbol.name
            }).unwrap_or("default".to_string());
        println!(
            "frame_addr: {:?}, relative_addr: {:?}, sym: {:?}",
            addr, relative_addr, symbol
        );
    }
}

fn read_memory(address: u64) -> u64 {
    // 指定されたメモリ位置から値を読み込む
    let value: usize;
    unsafe {
        value = *(address as *const usize);
    }
    value as u64
}

pub async fn backtrace(aslr_offset: u64, symbol_map: SymbolMap, unwinder: UnwinderX86_64<Vec<u8>>, cache: &mut CacheX86_64) {

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
        let _symbol = symbol_map
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
        Err(_) => panic!("boooooooon"),
    };
    let symbol_map: SymbolMap = symbol_manager
        .load_symbol_map_for_binary_at_path(&path, None)
        .await
        .unwrap();

    symbol_map
}