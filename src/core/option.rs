use generic_mutability::{gen_mut, GenRef, Mutability};

pub trait OptionExt<'s, M: Mutability, T> {
    fn as_ref_gen(self) -> Option<GenRef<'s, M, T>>;
}

impl<'s, M: Mutability, T> OptionExt<'s, M, T> for GenRef<'s, M, Option<T>> {
    fn as_ref_gen(self) -> Option<GenRef<'s, M, T>> {
        gen_mut!(M => {
            match from_gen!(self) {
                Some(x) => Some(into_gen!(x)),
                None => None
            }
        })
    }
}
