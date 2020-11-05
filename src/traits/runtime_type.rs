use crate::*;

/// RuntimeType is used to constrain WinRT generic types to WinRT types.
///
/// It is highly unlikely that users of WinRT will ever need to implement this
/// trait for themselves.
///
/// # Safety
///
/// A type should only implement RuntimeType if the associated `Abi` type is safe to pass
/// across FFI boundaries. The type itself must also be zero-initializable and safe to
/// drop if all bits are zeroable. RuntimeTypes must be safe to use in WinRT generics.
pub unsafe trait RuntimeType: Abi + IntoResult + Clone {
    const SIGNATURE: crate::ConstBuffer;
}

macro_rules! primitive_runtime_types {
    ($(($t:ty, $s:literal)),+) => {
        $(
            unsafe impl RuntimeType for $t {
                const SIGNATURE: crate::ConstBuffer = crate::ConstBuffer::from_slice($s);
            }
            unsafe impl Abi for $t {
                type Abi = Self;
                unsafe fn get_abi(&self) -> Self {
                    *self
                }
                unsafe fn set_abi(&mut self) -> *mut Self {
                    self
                }
            }
            unsafe impl IntoResult for $t {
                type Abi = Self;
                unsafe fn into_result(abi: Self::Abi) -> Result<Self> {
                    Ok(abi)
                }
            }
        )*
    };
}

primitive_runtime_types! {
    (bool, b"b1"),
    (i8, b"i1"),
    (u8, b"u1"),
    (i16, b"i2"),
    (u16, b"u2"),
    (i32, b"i4"),
    (u32, b"u4"),
    (i64, b"i8"),
    (u64, b"u8"),
    (f32, b"f4"),
    (f64, b"f8")
}
