// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
pub use bitfield_impl::{bitfield, BitfieldSpecifier};

pub trait Specifier {
    type Bucket;
    const BITS: usize;

    fn is_set(bucket: &Self::Bucket, bit: usize) -> bool;
    fn set(bucket: &mut Self::Bucket, bit: usize);
    fn empty() -> Self::Bucket;
}

macro_rules! b {
    ($ident: ident, $lit: literal, $bucket: ident) => {
        pub enum $ident {}
        impl Specifier for $ident {
            type Bucket = $bucket;
            const BITS: usize = $lit;

            fn is_set(bucket: &Self::Bucket, bit: usize) -> bool {
                bucket & (1 << bit) != 0
            }

            fn set(bucket: &mut Self::Bucket, bit: usize) {
                *bucket |= (1 << bit);
            }

            fn empty() -> Self::Bucket {
                0
            }
        }
    };
}

impl Specifier for bool {
    type Bucket = Self;
    const BITS: usize = 1;

    fn is_set(bucket: &Self::Bucket, _: usize) -> bool {
        *bucket
    }

    fn set(bucket: &mut Self::Bucket, _: usize) {
        *bucket = true;
    }

    fn empty() -> Self::Bucket {
        false
    }
}

b!(B1, 1, u8);
b!(B2, 2, u8);
b!(B3, 3, u8);
b!(B4, 4, u8);
b!(B5, 5, u8);
b!(B6, 6, u8);
b!(B7, 7, u8);
b!(B8, 8, u8);
b!(B9, 9, u16);

b!(B10, 10, u16);
b!(B11, 11, u16);
b!(B12, 12, u16);
b!(B13, 13, u16);
b!(B14, 14, u16);
b!(B15, 15, u16);
b!(B16, 16, u16);
b!(B17, 17, u32);
b!(B18, 18, u32);
b!(B19, 19, u32);

b!(B20, 20, u32);
b!(B21, 21, u32);
b!(B22, 22, u32);
b!(B23, 23, u32);
b!(B24, 24, u32);
b!(B25, 25, u32);
b!(B26, 26, u32);
b!(B27, 27, u32);
b!(B28, 28, u32);
b!(B29, 29, u32);

b!(B30, 30, u32);
b!(B31, 31, u32);
b!(B32, 32, u64);
b!(B33, 33, u64);
b!(B34, 34, u64);
b!(B35, 35, u64);
b!(B36, 36, u64);
b!(B37, 37, u64);
b!(B38, 38, u64);
b!(B39, 39, u64);

b!(B40, 40, u64);
b!(B41, 41, u64);
b!(B42, 42, u64);
b!(B43, 43, u64);
b!(B44, 44, u64);
b!(B45, 45, u64);
b!(B46, 46, u64);
b!(B47, 47, u64);
b!(B48, 48, u64);
b!(B49, 49, u64);

b!(B50, 50, u64);
b!(B51, 51, u64);
b!(B52, 52, u64);
b!(B53, 53, u64);
b!(B54, 54, u64);
b!(B55, 55, u64);
b!(B56, 56, u64);
b!(B57, 57, u64);
b!(B58, 58, u64);
b!(B59, 59, u64);

b!(B60, 60, u64);
b!(B61, 61, u64);
b!(B62, 62, u64);
b!(B63, 63, u64);
b!(B64, 64, u64);
// TODO other things
