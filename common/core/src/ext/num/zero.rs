pub trait Zero {
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
}

macro_rules! impl_zero {
    ($($t:ty),* $(,)?) => {
        $(
            impl Zero for $t {
                #[inline]
                fn zero() -> Self { 0 as $t }
                #[inline]
                fn is_zero(&self) -> bool { *self == 0 as $t }
            }
        )*
    };
}

impl_zero!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64,
);
