use std::fmt::Debug;

pub trait TrueOrExt {
    fn true_or<E: Debug>(self, e: E) -> Result<(), E>;
}
impl TrueOrExt for bool {
    fn true_or<E: Debug>(self, e: E) -> Result<(), E> {
        match self {
            true => Ok(()),
            false => Err(e),
        }
    }
}
