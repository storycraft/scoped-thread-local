fn main() {
    #[derive(Debug)]
    pub struct Context<'a, 'b> {
        pub a: &'a i32,
        pub b: &'b mut i32,
    }

    scoped_thread_local::scoped_thread_local!(pub static LOCAL: Context<'_, '_>);

    LOCAL.set(Context { a: &2, b: &mut 123 }, || {
        LOCAL.with(|cx| {
            // Prints `Context { a: 2, b: 123 }`
            println!("{cx:?}");
            *cx.b = 3;
        });

        LOCAL.with(|cx| {
            // Prints `Context { a: 2, b: 3 }`
            println!("{cx:?}");
        });
    });
}
