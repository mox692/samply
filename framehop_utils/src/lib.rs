use framehop::{
    x86_64::{CacheX86_64, UnwindRegsX86_64, UnwinderX86_64},
    FrameAddress, Unwinder,
};
use std::arch::asm;

/// 下記を実施する.
///
/// * libのload
/// * 実行ファイルのaslrのoffsetとかを取得する
/// * cacheとかunwinderの設定を調整する
struct BuilderX86_64 {}

impl BuilderX86_64 {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> StackUnwinderX86_64 {
        StackUnwinderX86_64 {
            cache: CacheX86_64::<_>::new(),
            unwinder: UnwinderX86_64::new(),
            closure: Box::new(|addr: u64| {
                // TODO: better impl
                if addr % 8 != 0 {
                    // Unaligned address
                    return Err(());
                }
                // SAFETY: TODO
                unsafe { Ok(*(addr as *const u64)) }
            }),
        }
    }
}
impl Default for BuilderX86_64 {
    fn default() -> Self {
        Self {}
    }
}

pub struct StackUnwinderX86_64 {
    cache: CacheX86_64,
    // TODO: update vec.
    unwinder: UnwinderX86_64<Vec<u8>>,
    closure: Box<dyn FnMut(u64) -> Result<u64, ()>>,
}

impl StackUnwinderX86_64 {
    pub fn unwind<'a>(&'a mut self) -> UnwindIterator<'a> {
        let (rip, regs) = {
            let mut rip = 0;
            let mut rsp = 0;
            let mut rbp = 0;
            unsafe { asm!("lea {}, [rip]", out(reg) rip) };
            unsafe { asm!("mov {}, rsp", out(reg) rsp) };
            unsafe { asm!("mov {}, rbp", out(reg) rbp) };
            (rip, UnwindRegsX86_64::new(rip, rsp, rbp))
        };

        let iter = self
            .unwinder
            .iter_frames(rip, regs, &mut self.cache, &mut self.closure);

        UnwindIterator::new(iter)
    }
}

pub struct UnwindIterator<'a> {
    inner: framehop::UnwindIterator<
        'a,
        'a,
        'a,
        UnwinderX86_64<Vec<u8>>,
        Box<dyn FnMut(u64) -> Result<u64, ()>>,
    >,
}

impl<'a> UnwindIterator<'a> {
    fn new(
        inner: framehop::UnwindIterator<
            'a,
            'a,
            'a,
            UnwinderX86_64<Vec<u8>>,
            Box<dyn FnMut(u64) -> Result<u64, ()>>,
        >,
    ) -> Self {
        Self { inner }
    }
}

// Should we expose FallibleIterator?
impl<'a> Iterator for UnwindIterator<'a> {
    type Item = FrameAddress;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().ok().flatten()
    }
}

// check basic usages.
#[test]
fn basic() {
    //
    // Usage
    //
    let mut unwinder = BuilderX86_64::new().build();

    let mut iter = unwinder.unwind();

    while let Some(frame) = iter.next() {
        println!("frame: {:?}", &frame)
    }
}
