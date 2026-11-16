use std::fmt::Debug;
use std::fmt::Display;

pub trait DisplayExt {
    fn displayln(&self);
}
pub trait DebugExt {
    fn debugln(&self);
}

impl<T: Display> DisplayExt for T {
    fn displayln(&self) {
        println!("{}", self);
    }
}

impl<T: Debug> DebugExt for T {
    fn debugln(&self) {
        println!("{:?}", self);
    }
}
