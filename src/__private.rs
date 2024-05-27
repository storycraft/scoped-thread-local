use core::cell::Cell;

use elain::{Align, Alignment};

pub const EMPTY_MESSAGE: &str = "expected thread local value, found none";

#[macro_export]
#[doc(hidden)]
macro_rules! generate {
    (
        $(#[$meta:meta])*
        [vis: $vis:vis]
        [name: $name:ident]
        [hkt_ty: for<$($lt:lifetime),*> $hkt_ty:ty]
    ) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        $(#[$meta])*
        $vis struct $name;

        const _: () = {
            const LAYOUT: ::core::alloc::Layout = {
                const fn get<$($lt),*>() -> ::core::alloc::Layout {
                    ::core::alloc::Layout::new::<$hkt_ty>()
                }

                get()
            };

            ::std::thread_local!(
                static INNER: ::core::cell::Cell<
                ::core::option::Option<
                    ::core::mem::MaybeUninit<
                        $crate::__private::Opaque<
                            { LAYOUT.size() },
                            { LAYOUT.align() }
                        >
                    >
                >
            > = const { ::core::cell::Cell::new(None) }
            );

            impl $name {
                /// Take value and insert into this scoped thread local storage slot for a duration of a closure.
                ///
                /// Upon return, this function will restore the previous value and return the taken value.
                #[inline(always)]
                pub fn set<$($lt),*>(&self, value: $hkt_ty, f: impl FnOnce()) -> $hkt_ty {
                    INNER.with(|inner| {
                        // SAFETY: extended lifetimes are not exposed and only accessible via higher kinded closure
                        let slot = ::core::cell::Cell::new(Some(unsafe { ::core::mem::transmute(value) }));

                        $crate::__private::with_swapped(inner, &slot, f);

                        // SAFETY: restore lifetimes
                        unsafe { ::core::mem::transmute(slot.into_inner().unwrap()) }
                    })
                }

                /// Temporary takes out value and obtain a mutable reference value.
                /// This function takes a closure which receives the value of this variable.
                ///
                /// # Panics
                /// Panics if set has not previously been called or value is already taken out by parent scope.
                #[inline(always)]
                pub fn with<R>(self, f: impl for<$($lt),*> FnOnce(&mut $hkt_ty) -> R) -> R {
                    INNER.with(|cell| {
                        $crate::__private::with_key(
                            cell,
                            // SAFETY: opaque type has same layout
                            |opt| f(unsafe {
                                ::core::mem::transmute(
                                    opt.as_mut().expect($crate::__private::EMPTY_MESSAGE)
                                )
                            })
                        )
                    })
                }

                /// Test whether this TLS key has been set for the current thread.
                #[inline(always)]
                pub fn is_set(self) -> bool {
                    INNER.with(|cell| {
                        $crate::__private::with_key(
                            cell,
                            |opt| opt.is_some()
                        )
                    })
                }
            }
        };
    };
}

pub fn with_swapped<T>(cell1: &Cell<T>, cell2: &Cell<T>, f: impl FnOnce()) {
    struct Guard<'a, T> {
        cell1: &'a Cell<T>,
        cell2: &'a Cell<T>,
    }

    impl<T> Drop for Guard<'_, T> {
        fn drop(&mut self) {
            self.cell1.swap(self.cell2);
        }
    }

    cell1.swap(cell2);
    let _guard = Guard { cell1, cell2 };

    f()
}

pub fn with_key<T, R>(cell: &Cell<Option<T>>, f: impl FnOnce(&mut Option<T>) -> R) -> R {
    struct Guard<'a, T> {
        cell: &'a Cell<Option<T>>,
        value: Option<T>,
    }

    impl<T> Drop for Guard<'_, T> {
        fn drop(&mut self) {
            self.cell.set(self.value.take());
        }
    }

    f(&mut Guard {
        cell,
        value: cell.take(),
    }
    .value)
}

#[repr(C)]
pub struct Opaque<const SIZE: usize, const ALIGN: usize>([u8; SIZE], Align<ALIGN>)
where
    Align<ALIGN>: Alignment;
