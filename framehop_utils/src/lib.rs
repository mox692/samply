
/// 
pub fn backtrace() {

}


#[test]
fn sketch() {
    /// 下記を実施する.
    ///
    /// * libのload
    /// * 実行ファイルのaslrのoffsetとかを取得する
    struct Builder {}

    impl Builder {
        fn new() -> Self {
            Self::default()
        }
        fn build(self) -> StackUnwinder{
            StackUnwinder {
            }
        }
    }
    impl Default for Builder {
        fn default() -> Self {
            Self {
            }
        }
    }

    struct StackUnwinder {
    };

    struct UnwindIterator {
    };

    impl StackUnwinder {
        fn unwind(&self) -> UnwindIterator {
            let rip = todo!();
            let regs = todo!();
            let cache = todo!();
            let read_stack = todo!();
            // self.inner.iter_frames(rip, regs, &mut cache, todo!());

            UnwindIterator {}
        }
    }


    //
    // Usage
    //
    let unwinder = Builder::new().build();

    let iter = unwinder.unwind();


}