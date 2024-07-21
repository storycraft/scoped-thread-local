use std::marker::PhantomData;

use scoped_thread_local::scoped_thread_local;

#[derive(Debug)]
pub struct Container<'a, T1, T2, const A: usize> {
    pub a: Option<&'a mut i32>,
    pub b: T1,
    pub c: T2,
    pub d: PhantomData<[u8; A]>,
}

impl<T1, T2, const A: usize> Drop for Container<'_, T1, T2, A> {
    fn drop(&mut self) {
        println!("value dropped");
    }
}

scoped_thread_local!(
    pub static EXAMPLE: Container<&i32, Vec<i32>, 271>
);

fn main() {
    let mut a = 7;
    let mut a = Container {
        a: Some(&mut a),
        b: &2,
        c: vec![1],
        d: PhantomData,
    };

    EXAMPLE.set(&mut a, || {
        EXAMPLE.with(|inner| {
            dbg!(&inner);
            inner.b = &1;
        });

        EXAMPLE.with(|inner| {
            dbg!(inner);
        });
    });

    dbg!(a);
}
