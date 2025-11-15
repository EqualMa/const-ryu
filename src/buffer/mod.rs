use crate::raw;
use core::mem::MaybeUninit;
use core::{slice, str};
#[cfg(feature = "no-panic")]
use no_panic::no_panic;

const NAN: &str = "NaN";
const INFINITY: &str = "inf";
const NEG_INFINITY: &str = "-inf";

/// Safe API for formatting floating point numbers to text.
///
/// ## Example
///
/// ```
/// let mut buffer = const_ryu::Buffer::new();
/// let printed = buffer.format_finite(1.234);
/// assert_eq!(printed, "1.234");
/// ```
pub struct Buffer {
    bytes: [MaybeUninit<u8>; 24],
}

impl Buffer {
    /// This is a cheap operation; you don't need to worry about reusing buffers
    /// for efficiency.
    #[inline]
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub const fn new() -> Self {
        let bytes = [MaybeUninit::<u8>::uninit(); 24];
        Buffer { bytes }
    }

    /// Print a floating point number into this buffer and return a reference to
    /// its string representation within the buffer.
    ///
    /// # Special cases
    ///
    /// This function formats NaN as the string "NaN", positive infinity as
    /// "inf", and negative infinity as "-inf" to match std::fmt.
    ///
    /// If your input is known to be finite, you may get better performance by
    /// calling the `format_finite` method instead of `format` to avoid the
    /// checks for special cases.
    #[cfg_attr(feature = "no-panic", inline)]
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub fn format<F: Float>(&mut self, f: F) -> &str {
        if f.is_nonfinite() {
            f.format_nonfinite()
        } else {
            self.format_finite(f)
        }
    }

    /// Print a floating point number into this buffer and return a reference to
    /// its string representation within the buffer.
    ///
    /// # Special cases
    ///
    /// This function **does not** check for NaN or infinity. If the input
    /// number is not a finite float, the printed representation will be some
    /// correctly formatted but unspecified numerical value.
    ///
    /// Please check [`is_finite`] yourself before calling this function, or
    /// check [`is_nan`] and [`is_infinite`] and handle those cases yourself.
    ///
    /// [`is_finite`]: f64::is_finite
    /// [`is_nan`]: f64::is_nan
    /// [`is_infinite`]: f64::is_infinite
    #[inline]
    #[cfg_attr(feature = "no-panic", no_panic)]
    pub fn format_finite<F: Float>(&mut self, f: F) -> &str {
        #[expect(clippy::ptr_as_ptr)]
        unsafe {
            let n = f.write_to_ryu_buffer(self.bytes.as_mut_ptr() as *mut u8);
            debug_assert!(n <= self.bytes.len());
            let slice = slice::from_raw_parts(self.bytes.as_ptr() as *const u8, n);
            str::from_utf8_unchecked(slice)
        }
    }
}

impl Copy for Buffer {}

#[allow(clippy::non_canonical_clone_impl)]
impl Clone for Buffer {
    #[inline]
    fn clone(&self) -> Self {
        Buffer::new()
    }
}

impl Default for Buffer {
    #[inline]
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn default() -> Self {
        Buffer::new()
    }
}

/// A floating point number, f32 or f64, that can be written into a
/// [`const_ryu::Buffer`][Buffer].
///
/// This trait is sealed and cannot be implemented for types outside of the
/// `ryu` crate.
pub trait Float: Sealed {}
impl Float for f32 {}
impl Float for f64 {}

pub trait Sealed: Copy {
    fn is_nonfinite(self) -> bool;
    fn format_nonfinite(self) -> &'static str;
    unsafe fn write_to_ryu_buffer(self, result: *mut u8) -> usize;
}

#[derive(Debug, Clone, Copy)]
struct ConstFloat<T>(T);

impl ConstFloat<f32> {
    const fn to_bits(self) -> u32 {
        self.0.to_bits()
    }
}
impl ConstFloat<f64> {
    const fn to_bits(self) -> u64 {
        self.0.to_bits()
    }
}

macro_rules! Float {
    ($(
        impl $Trait:ident for $Ty:ty {$(
            $(#$attr:tt)*
            $f1:ident $f2:ident $($f3:ident)? ($this:ident $(, $rest_arg:ident: $RestArgType:ty)* $(,)?) -> $ReturnType:ty
            $f_body:block
        )*}
    )*) => {$(
        impl ConstFloat<$Ty> {$(
            #[inline] // Is this good?
            const $f1 $f2 $($f3)? ($this $(, $rest_arg: $RestArgType)*) -> $ReturnType
            $f_body
        )*}
        impl $Trait for $Ty {$(
            $(#$attr)* // Is this good?
            $f1 $f2 $($f3)? ($this $(, $rest_arg: $RestArgType)*) -> $ReturnType {
                Float!(@call_fn (ConstFloat::<$Ty>($this)) ($f2 $($f3)?) ($($rest_arg),*))
            }
        )*}
    )*};
    (@call_fn ($this:expr) ($f:ident) $args:tt) => {
        $this.$f $args
    };
    (@call_fn ($this:expr) (fn $f:ident) $args:tt) => {
        $this.$f $args
    };
}

Float! {
impl Sealed for f32 {
    #[inline]
    fn is_nonfinite(self) -> bool {
        const EXP_MASK: u32 = 0x7f800000;
        let bits = self.to_bits();
        bits & EXP_MASK == EXP_MASK
    }

    #[cold]
    #[cfg_attr(feature = "no-panic", inline)]
    fn format_nonfinite(self) -> &'static str {
        const MANTISSA_MASK: u32 = 0x007fffff;
        const SIGN_MASK: u32 = 0x80000000;
        let bits = self.to_bits();
        if bits & MANTISSA_MASK != 0 {
            NAN
        } else if bits & SIGN_MASK != 0 {
            NEG_INFINITY
        } else {
            INFINITY
        }
    }

    #[inline]
    unsafe fn write_to_ryu_buffer(self, result: *mut u8) -> usize {
        raw::format32(self.0, result)
    }
}

impl Sealed for f64 {
    #[inline]
    fn is_nonfinite(self) -> bool {
        const EXP_MASK: u64 = 0x7ff0000000000000;
        let bits = self.to_bits();
        bits & EXP_MASK == EXP_MASK
    }

    #[cold]
    #[cfg_attr(feature = "no-panic", inline)]
    fn format_nonfinite(self) -> &'static str {
        const MANTISSA_MASK: u64 = 0x000fffffffffffff;
        const SIGN_MASK: u64 = 0x8000000000000000;
        let bits = self.to_bits();
        if bits & MANTISSA_MASK != 0 {
            NAN
        } else if bits & SIGN_MASK != 0 {
            NEG_INFINITY
        } else {
            INFINITY
        }
    }

    #[inline]
    unsafe fn write_to_ryu_buffer(self, result: *mut u8) -> usize {
        raw::format64(self.0, result)
    }
}
}

/// Const & safe API for formatting floating point numbers to text.
///
/// ## Example
///
/// ```
/// # const _: () = {macro_rules! assert_eq {($a:expr,$b:expr)=>{{mod __b{pub const VAL:&[u8]=$b.as_bytes();}assert!(matches!($a.as_bytes(), __b::VAL))}}}
/// let mut buffer = const_ryu::Buffer::new();
///
/// let printed = const_ryu::Format(&mut buffer, 1.234f32).call_once();
/// assert_eq!(printed, "1.234");
///
/// let printed = const_ryu::Format(&mut buffer, 1.234f64).call_once();
/// assert_eq!(printed, "1.234");
///
/// let printed = const_ryu::Format(&mut buffer, f64::NAN).call_once();
/// assert_eq!(printed, "NaN");
/// # };
/// ```
pub struct Format<'a, T>(pub &'a mut Buffer, pub T);

/// Const & safe API for formatting finite floating point numbers to text.
///
/// ## Example
///
/// ```
/// # const _: () = {macro_rules! assert_eq {($a:expr,$b:expr)=>{{mod __b{pub const VAL:&[u8]=$b.as_bytes();}assert!(matches!($a.as_bytes(), __b::VAL))}}}
/// let mut buffer = const_ryu::Buffer::new();
///
/// let printed = const_ryu::FormatFinite(&mut buffer, 1.234f32).call_once();
/// assert_eq!(printed, "1.234");
///
/// let printed = const_ryu::FormatFinite(&mut buffer, 1.234f64).call_once();
/// assert_eq!(printed, "1.234");
/// # };
/// ```
pub struct FormatFinite<'a, T>(pub &'a mut Buffer, pub T);

macro_rules! impl_fns {
    (
        type $T:ident = each_of!$each_of:tt;

        $($rest:tt)*
    ) => {
        impl_fns! { @impl $T $each_of {$($rest)*} }
    };
    (@impl $T:ident [$($Ty:ty),* $(,)?] $impl_body:tt) => {$(
        const _: () = {
            type $T = $Ty;
            const _: () = $impl_body;
        };
    )*};
}

impl_fns!(
    type F = each_of![f32, f64];

    impl<'a> Format<'a, F> {
        pub const fn call_once(self) -> &'a str {
            let Self(this, f) = self;
            let f = ConstFloat(f);

            if f.is_nonfinite() {
                f.format_nonfinite()
            } else {
                FormatFinite(this, f.0).call_once()
            }
        }
    }

    impl<'a> FormatFinite<'a, F> {
        pub const fn call_once(self) -> &'a str {
            let Self(this, f) = self;
            let f = ConstFloat(f);

            #[expect(clippy::ptr_as_ptr)]
            unsafe {
                let n = f.write_to_ryu_buffer(this.bytes.as_mut_ptr() as *mut u8);
                debug_assert!(n <= this.bytes.len());
                let slice = slice::from_raw_parts(this.bytes.as_ptr() as *const u8, n);
                str::from_utf8_unchecked(slice)
            }
        }
    }
);
