use core::fmt;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;

use generic_mutability::{gen_mut, GenRef, Mutability};

pub trait SliceExt<'s, M: Mutability, T> {
    fn iter_gen(self) -> IterGen<'s, M, T>;
}

impl<'s, M: Mutability, T> SliceExt<'s, M, T> for GenRef<'s, M, [T]> {
    fn iter_gen(self) -> IterGen<'s, M, T> {
        IterGen::new(self)
    }
}

// INVARIANT: the `IterGenInner` value must match the `M` mutability parameter
pub struct IterGen<'s, M: Mutability, T> {
    _mutability: PhantomData<*const M>,
    iter: IterGenInner<'s, T>,
}

union IterGenInner<'s, T> {
    shared: ManuallyDrop<core::slice::Iter<'s, T>>,
    mutable: ManuallyDrop<core::slice::IterMut<'s, T>>,
}

impl<M: Mutability, T: fmt::Debug> fmt::Debug for IterGen<'_, M, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IterGen").field(&self.as_slice()).finish()
    }
}

unsafe impl<M: Mutability, T: Sync> Sync for IterGen<'_, M, T> {}
unsafe impl<M: Mutability, T: Send> Send for IterGen<'_, M, T> {}

impl<'a, M: Mutability, T> IterGen<'a, M, T> {
    #[inline]
    pub(crate) fn new(slice: GenRef<'a, M, [T]>) -> Self {
        gen_mut!(M => {
            let iter = switch_shared_mut!(<[_]>::iter, <[_]>::iter_mut)(from_gen!(slice));
            let md_iter = ManuallyDrop::new(iter);

            Self{
                _mutability: PhantomData,
                // SAFETY: we are writing to the field matching `M`
                iter: switch_shared_mut!(IterGenInner{ shared: md_iter }, IterGenInner{ mutable: md_iter })
            }
        })
    }

    //TODO: Rewrite docs
    /// Views the underlying data as a subslice of the original data.
    ///
    /// To avoid creating `&mut` references that alias, this is forced
    /// to consume the iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// // First, we declare a type which has `iter_mut` method to get the `IterGen`
    /// // struct (`&[usize]` here):
    /// let mut slice = &mut [1, 2, 3];
    ///
    /// {
    ///     // Then, we get the iterator:
    ///     let mut iter = slice.iter_mut();
    ///     // We move to next element:
    ///     iter.next();
    ///     // So if we print what `into_slice` method returns here, we have "[2, 3]":
    ///     println!("{:?}", iter.into_slice());
    /// }
    ///
    /// // Now let's modify a value of the slice:
    /// {
    ///     // First we get back the iterator:
    ///     let mut iter = slice.iter_mut();
    ///     // We change the value of the first element of the slice returned by the `next` method:
    ///     *iter.next().unwrap() += 1;
    /// }
    /// // Now slice is "[2, 2, 3]":
    /// println!("{slice:?}");
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_slice(self) -> GenRef<'a, M, [T]> {
        gen_mut!(M => {
            // SAFETY: Invariants guarantee that we are reading the same field that we have written to
            let md_iter = unsafe{ switch_shared_mut!(self.iter.shared, self.iter.mutable) };

            let iter = ManuallyDrop::into_inner(md_iter);

            into_gen!(switch_shared_mut!(iter.as_slice(), iter.into_slice()))
        })
    }

    // TODO: Rewrite docs
    /// Views the underlying data as a subslice of the original data.
    ///
    /// To avoid creating `&mut [T]` references that alias, the returned slice
    /// borrows its lifetime from the iterator the method is applied on.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut slice: &mut [usize] = &mut [1, 2, 3];
    ///
    /// // First, we get the iterator:
    /// let mut iter = slice.iter_mut();
    /// // So if we check what the `as_slice` method returns here, we have "[1, 2, 3]":
    /// assert_eq!(iter.as_slice(), &[1, 2, 3]);
    ///
    /// // Next, we move to the second element of the slice:
    /// iter.next();
    /// // Now `as_slice` returns "[2, 3]":
    /// assert_eq!(iter.as_slice(), &[2, 3]);
    /// ```
    #[must_use]
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        gen_mut!(M => {
            // SAFETY: Invariants guarantee that we are reading the same field that we have written to
            let md_iter = unsafe { &switch_shared_mut!(self.iter.shared, self.iter.mutable) };

            md_iter.as_slice()
        })
    }
}

impl<M: Mutability, T> AsRef<[T]> for IterGen<'_, M, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'s, M: Mutability, T> Iterator for IterGen<'s, M, T> {
    type Item = GenRef<'s, M, T>;
    fn next(&mut self) -> Option<Self::Item> {
        gen_mut!(M => {
            // SAFETY: Invariants guarantee that we are reading the same field that we have written to
            let md_iter = unsafe { &mut switch_shared_mut!(self.iter.shared, self.iter.mutable) };

            md_iter.next().map(into_gen!())
        })
    }
}

impl<'s, M: Mutability, T> DoubleEndedIterator for IterGen<'s, M, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        gen_mut!(M => {
            // SAFETY: Invariants guarantee that we are reading the same field that we have written to
            let md_iter = unsafe { &mut switch_shared_mut!(self.iter.shared, self.iter.mutable) };

            md_iter.next_back().map(into_gen!())
        })
    }
}

impl<'s, M: Mutability, T> ExactSizeIterator for IterGen<'s, M, T> {
    fn len(&self) -> usize {
        gen_mut!(M => {
            // SAFETY: Invariants guarantee that we are reading the same field that we have written to
            let md_iter = unsafe { &switch_shared_mut!(self.iter.shared, self.iter.mutable) };

            md_iter.len()
        })
    }
}

impl<'s, M: Mutability, T> FusedIterator for IterGen<'s, M, T> {}

impl<'s, M: Mutability, T> Default for IterGen<'s, M, T> {
    fn default() -> Self {
        Self::new(GenRef::gen_from_mut_downgrading(&mut []))
    }
}
