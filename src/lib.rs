#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
//! ## Example
//! See `example` module for expanded code
//! ```rust
#![doc = include_str!("./example.rs")]
//! ```

#[doc(hidden)]
pub mod __private;

/// Expanded example code
#[cfg(doc)]
#[cfg_attr(docsrs, doc(cfg(feature = "example")))]
pub mod example;

#[macro_export]
/// Create scoped thread local
macro_rules! scoped_thread_local {
    () => {};

    (
        $(#[$meta:meta])*
        $vis:vis static $name:ident: for<$($lt:lifetime),*> $ty:ty $(; $($rest:tt)*)?
    ) => {
        $crate::generate!(
            $(#[$meta])*
            [vis: $vis]
            [name: $name]
            [hkt_ty: for<$($lt),*> $ty]
        );

        $($crate::scoped_thread_local!($($rest)*))?
    };

    (
        $(#[$meta:meta])*
        $vis:vis static $name:ident: $ty:ty $(; $($rest:tt)*)?
    ) => {
        $crate::scoped_thread_local!(
            $(#[$meta])*
            $vis static $name: for<> $ty
        );

        $($crate::scoped_thread_local!($($rest)*))?
    };
}
