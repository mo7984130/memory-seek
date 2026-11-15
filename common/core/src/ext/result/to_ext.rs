pub trait ToOkExt<T, E> {
    fn to_ok(self) -> Result<T, E>;
}
pub trait ToErrExt<T, E> {
    fn to_err(self) -> Result<T, E>;
}

impl<T, E> ToOkExt<T, E> for T {
    #[inline]
    fn to_ok(self) -> Result<T, E> {
        Ok(self)
    }
}
impl<T, E> ToErrExt<T, E> for E {
    #[inline]
    fn to_err(self) -> Result<T, E> {
        Err(self)
    }
}
