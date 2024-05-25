/// Container type for scoped thread local variable
pub struct Container<'a, 'b> {
    pub a: &'a mut i32,
    pub b: &'b i32,
}

crate::scoped_thread_local!(
    /// Generated scoped thread local variable.
    /// 
    /// Elided lifetimes are supported.
    pub static EXAMPLE: for<'a> Container<'a, '_>
);
