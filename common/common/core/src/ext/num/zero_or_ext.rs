use crate::ext::num::zero::Zero;

pub trait ZeroOrExt<T, E> {
    fn zero_or(self, e: E) -> Result<T, E>
    where
        T: Zero + PartialEq;
}

impl<T, E> ZeroOrExt<T, E> for T
where
    T: Zero + PartialEq,
{
    fn zero_or(self, e: E) -> Result<T, E> {
        if self.is_zero() { Err(e) } else { Ok(self) }
    }
}
