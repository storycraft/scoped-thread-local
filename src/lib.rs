#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod __private;

#[cfg(doc)]
pub mod example {
    //! Usage example for scoped thread local

    /// Container type for scoped thread local variable
    pub struct Container<'a, 'b> {
        pub a: &'a mut i32,
        pub b: &'b i32,
    }

    crate::scoped_thread_local!(
        /// Generated scoped thread local variable
        pub static EXAMPLE: for<'a> Container<'a, '_>
    );
}

#[macro_export]
/// Create scoped thread local
macro_rules! scoped_thread_local {
    (
        $(#[$meta:meta])*
        $vis:vis static $name:ident: for<$($lt:lifetime),*> $($ty_tt:tt)*
    ) => {
        $crate::generate!(
            $(#[$meta])*
            [vis: $vis]
            [name: $name]
            [hkt_ty: for<$($lt),*> $($ty_tt)*]
            [static_ty: $crate::staticify!(
                [input: $($ty_tt)*]
                [output: ]
            )]
        );
    };

    (
        $(#[$meta:meta])*
        $vis:vis static $name:ident: $($ty_tt:tt)*
    ) => {
        $crate::scoped_thread_local!(
            $(#[$meta])*
            $vis static $name: for<> $($ty_tt)*
        );
    };
}
