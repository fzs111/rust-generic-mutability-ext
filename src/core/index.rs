use core::ops::{Index, IndexMut};

use generic_mutability::{gen_mut, GenRef, Mutability};

pub trait IndexGen<'s, M: Mutability, I, T: Index<I> + IndexMut<I>> {
    fn index_gen(self, idx: I) -> GenRef<'s, M, T::Output>;
}

impl<'s, M: Mutability, I, T: Index<I> + IndexMut<I>> IndexGen<'s, M, I, T> for GenRef<'s, M, T> {
    fn index_gen(self, idx: I) -> GenRef<'s, M, T::Output> {
        gen_mut!(M => {
            into_gen!(&gen from_gen!(self)[idx])
        })
    }
}
