#![cfg(test)]

use generic_mutability::{GenRef, GenRefMethods};
use generic_mutability_ext::core::{index::IndexGen, slice::SliceExt};

#[test]
fn slice_iter_gen() {
    let mut a = vec![1, 2, 3, 4];

    let b = GenRef::from(&a);

    assert_eq!(b.index_gen(2), GenRef::from(&3));

    let mut iter = b.map_deref().iter_gen();

    assert_eq!(iter.next(), Some(GenRef::from(&1)));

    assert_eq!(iter.collect::<Vec<_>>(), vec![&2, &3, &4]);

    let mut b = GenRef::from(&mut a);

    assert_eq!(b.reborrow().index_gen(2), GenRef::from(&mut 3));

    *b.reborrow().index_gen(0) = 11;

    let mut iter = b.map_deref().iter_gen();

    assert_eq!(iter.next(), Some(GenRef::from(&mut 11)));

    assert_eq!(iter.collect::<Vec<_>>(), vec![&mut 2, &mut 3, &mut 4]);
}
