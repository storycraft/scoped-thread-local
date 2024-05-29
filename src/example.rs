/// Container type for scoped thread local variable
pub struct Container<'a, 'b> {
    pub a: &'a mut i32,
    pub b: &'b i32,
}

crate::scoped_thread_local!(
    /// Generated scoped thread local variable.
    /// 
    /// Elided lifetimes are also supported.
    pub static EXAMPLE: for<'a> Container<'a, '_>
);

fn main() {
    EXAMPLE.set(
        Container {
            a: &mut 1,
            b: &2,
        },
        || {
            EXAMPLE.with(|inner| {
                // Prints 3
                println!("{}", *inner.a + *inner.b);
            });
        },
    );
}
