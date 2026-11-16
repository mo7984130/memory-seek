use crate::ext::num::zero::Zero;

pub trait NonzeroOrExt<T, E> {
    fn zero_or(self, e: E) -> Result<T, E>
    where
        T: Zero + PartialEq;
}

impl<T, E> NonzeroOrExt<T, E> for T
where
    T: Zero + PartialEq,
{
    fn zero_or(self, e: E) -> Result<T, E> {
        if self.is_zero() { Ok(self) } else { Err(e) }
    }
}
