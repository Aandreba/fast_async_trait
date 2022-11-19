#![feature(unboxed_closures, fn_traits)]

extern crate fast_async_trait_proc;
pub use fast_async_trait_proc::*;

#[doc(hidden)]
pub trait FnOnceHelper {
    type Args;
    type Output;

    extern "rust-call" fn call_once(self, args: Self::Args) -> Self::Output;
}