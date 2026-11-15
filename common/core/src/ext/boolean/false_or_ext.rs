use std::fmt::Debug;

pub trait FalseOrExt {
    fn false_or<E: Debug>(self, e: E) -> Result<bool, E>;
}
impl FalseOrExt for bool {
    fn false_or<E: Debug>(self, e: E) -> Result<bool, E> {
        match self {
            false => Ok(true),
            true => Err(e),
        }
    }
}
