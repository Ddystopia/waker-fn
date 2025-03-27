//! Convert closures into wakers.
//!
//! A [`Waker`] is just a fancy callback. This crate converts regular closures into wakers.

#![no_std]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/smol-rs/smol/master/assets/images/logo_fullsize_transparent.png"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/smol-rs/smol/master/assets/images/logo_fullsize_transparent.png"
)]

#[cfg(all(not(feature = "portable-atomic"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "portable-atomic"), feature = "alloc"))]
use alloc::{sync::Arc, task::Wake};
use core::task::Waker;
#[cfg(all(feature = "portable-atomic", feature = "alloc"))]
use portable_atomic_util::{task::Wake, Arc};

/// Converts a closure into a [`Waker`].
///
/// The closure gets called every time the waker is woken.
///
/// # Examples
///
/// ```
/// use waker_fn::waker_fn;
///
/// let waker = waker_fn(|| println!("woken"));
///
/// waker.wake_by_ref(); // Prints "woken".
/// waker.wake();        // Prints "woken".
/// ```
pub fn waker_fn<F: Fn() + Send + Sync + 'static>(f: F) -> Waker {
    Waker::from(Arc::new(Helper(f)))
}

struct Helper<F>(F);

#[cfg(all(not(feature = "portable-atomic"), feature = "alloc"))]
impl<F: Fn() + Send + Sync + 'static> Wake for Helper<F> {
    fn wake(self: Arc<Self>) {
        (self.0)();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        (self.0)();
    }
}
// Note: Unlike std::task::Wake, all methods take `this:` instead of `self:`.
// This is because using portable_atomic_util::Arc as a receiver requires the
// unstable arbitrary_self_types feature.
#[cfg(all(feature = "portable-atomic", feature = "alloc"))]
impl<F: Fn() + Send + Sync + 'static> Wake for Helper<F> {
    fn wake(this: Arc<Self>) {
        (this.0)();
    }

    fn wake_by_ref(this: &Arc<Self>) {
        (this.0)();
    }
}

pub const fn waker_fn_ptr(f: fn()) -> Waker {
    use core::mem::transmute;
    use core::task::{RawWaker, RawWakerVTable, Waker};

    static VTABLE: RawWakerVTable = unsafe {
        RawWakerVTable::new(
            |this| RawWaker::new(this, &VTABLE),
            |this| transmute::<*const (), fn()>(this)(),
            |this| transmute::<*const (), fn()>(this)(),
            |_| {},
        )
    };
    let raw = RawWaker::new(f as *const (), &VTABLE);

    unsafe { Waker::from_raw(raw) }
}
