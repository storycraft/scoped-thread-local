pub use scopeguard;

pub const EMPTY_MESSAGE: &str = "empty";

#[macro_export]
#[doc(hidden)]
macro_rules! into_tt {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

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
                pub fn set<$($lt),*>(&self, value: $hkt_ty, f: impl FnOnce()) -> $hkt_ty {
                    // SAFETY: extended lifetimes are not exposed and only accessible via higher kinded closure
                    let slot = ::core::cell::Cell::new(Some(unsafe { ::core::mem::transmute(value) }));

                    INNER.with(|inner| {
                        inner.swap(&slot);
                        {
                            $crate::__private::scopeguard::defer! {
                                inner.swap(&slot);
                            };

                            f();
                        }

                        // SAFETY: restore lifetimes
                        unsafe { ::core::mem::transmute(slot.into_inner().unwrap()) }
                    })
                }

                /// Temporary takes out value and obtain a mutable reference value.
                /// This function takes a closure which receives the value of this variable.
                /// 
                /// # Panics
                /// Panics if set has not previously been called or value is already taken out by parent scope.
                pub fn with<R>(self, f: impl for<$($lt),*> FnOnce(&mut $hkt_ty) -> R) -> R {
                    INNER.with(|inner| {
                        let mut val = $crate::__private::scopeguard::guard(
                            inner.take().expect($crate::__private::EMPTY_MESSAGE),
                            |val| inner.set(Some(val)),
                        );

                        f(&mut *val)
                    })
                }

                /// Test whether this TLS key has been set for the current thread.
                pub fn is_set(self) -> bool {
                    INNER.with(|inner| {
                        let mut opt = $crate::__private::scopeguard::guard(
                            inner.take(),
                            |opt| inner.set(opt),
                        );

                        opt.is_some()
                    })
                }
            }
        };
    };
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
