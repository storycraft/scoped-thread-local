use std::{cell::Cell, thread::LocalKey};

pub const EMPTY_MESSAGE: &str = "expected thread local value, found none";

#[macro_export]
#[doc(hidden)]
macro_rules! generate {
    (
        $(#[$meta:meta])*
        [vis: $vis:vis]
        [name: $name:ident]
        [hkt_ty: for<$($lt:lifetime),*> $hkt_ty:ty]
        [static_ty: $static_ty:ty]
    ) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        $(#[$meta])*
        $vis struct $name;

        const _: () = {
            ::std::thread_local!(
                static INNER: ::core::cell::Cell<
                    ::core::option::Option<$static_ty>
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
                    $crate::__private::with_key(&INNER, |opt| f(opt.as_mut().expect($crate::__private::EMPTY_MESSAGE)))
                }

                /// Test whether this TLS key has been set for the current thread.
                #[inline(always)]
                pub fn is_set(self) -> bool {
                    $crate::__private::with_key(&INNER, |opt| opt.is_some())
                }
            }
        };
    };
}

pub fn with_swapped<T>(cell1: &Cell<T>, cell2: &Cell<T>, f: impl FnOnce()) {
    cell1.swap(cell2);
    scopeguard::defer! {
        cell1.swap(cell2);
    };

    f()
}

pub fn with_key<T, R>(
    key: &'static LocalKey<Cell<Option<T>>>,
    f: impl FnOnce(&mut Option<T>) -> R,
) -> R {
    key.with(|inner| f(&mut scopeguard::guard(inner.take(), |opt| inner.set(opt))))
}

#[macro_export]
#[doc(hidden)]
/// Convert every lifetimes to 'static
macro_rules! staticify {
    // Handle 'lt'
    (
        [input: $in_lt:lifetime $($in:tt)*]
        [output: $($out:tt)*]
    ) => {
        $crate::staticify!(
            [input: $($in)*]
            [output: $($out)* 'static]
        )
    };

    // Handle &'lt
    (
        [input: & $in_lt:lifetime $($in:tt)*]
        [output: $($out:tt)*]
    ) => {
        $crate::staticify!(
            [input: $($in)*]
            [output: &'static $($out)*]
        )
    };

    // Handle &
    (
        [input: & $($in:tt)*]
        [output: $($out:tt)*]
    ) => {
        $crate::staticify!(
            [input: $($in)*]
            [output: &'static $($out)*]
        )
    };

    // Handle paren
    (
        [input: ( $($in_paren:tt)+ ) $($in_rest:tt)*]
        [output: $($out:tt)*]
    ) => {
        $crate::staticify!(
            [input: $($in_rest)*]
            [output: $($out)* ( $crate::staticify!(
                [input: $($in_paren)*]
                [output: ]
            ) )]
        )
    };

    // Handle bracket
    (
        [input: [ $($in_bracket:tt)+ ] $($in_rest:tt)*]
        [output: $($out:tt)*]
    ) => {
        $crate::staticify!(
            [input: $($in_rest)*]
            [output: $($out)* [$crate::staticify!(
                [input: $($in_bracket)*]
                [output: ]
            )] ]
        )
    };

    // Forward otherwise
    (
        [input: $in:tt $($in_rest:tt)*]
        [output: $($out:tt)*]
    ) => {
        $crate::staticify!(
            [input: $($in_rest)*]
            [output: $($out)* $in]
        )
    };

    (
        [input: ]
        [output: $ty:ty]
    ) => {
        $ty
    };

    // Create error if output is not valid
    (
        [input: ]
        [output: $($tt:tt)*]
    ) => {
        ::core::compile_error!(::core::concat!("Expected type, output: ", ::core::stringify!($($tt)*)))
    };
}
