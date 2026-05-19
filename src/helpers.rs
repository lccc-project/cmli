use core::marker::PhantomData;
use std::fmt::{DebugAsHex, FormattingOptions};

use crate::fmt::{PrettyPrint, PrettyPrinter, pretty_print_list};



pub const trait BitsetTy: Copy {
    fn from_u32(bit: u32) -> Self;
    fn into_u32(self) -> u32;
}

struct ForceHexPrint(u64);

impl core::fmt::Debug for ForceHexPrint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(val) = self;
        match f.options().get_debug_as_hex() {
            Some(DebugAsHex::Upper) => f.write_fmt(format_args!("{val:#018X}")),
            _ => f.write_fmt(format_args!("{val:#018x}"))
        }
        
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Bitset<Ty, const N: usize>([u64; N], PhantomData<[Ty]>);

impl<Ty, const N: usize> core::fmt::Debug for Bitset<Ty, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(
            "Bitset"
        ).field_with(|f| {
            f.debug_list()
                .entries(self.0.iter().copied().map(ForceHexPrint))
                .finish()
        })
        .finish_non_exhaustive()
    }
}

impl<'a, Ty: BitsetTy + PrettyPrint, const N: usize> core::fmt::Display for PrettyPrinter<'a, Bitset<Ty, N>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pretty_print_list(*self.0, " ", self.1, self.2).fmt(f)
    }
}

impl<Ty, const N: usize> Bitset<Ty, N> {
    pub const fn new() -> Self {
        Self([0u64; N], PhantomData)
    }
}

impl<Ty: BitsetTy, const N: usize> Bitset<Ty, N> {
    pub const fn insert_bit(&mut self, bit: Ty) where Ty: [const] BitsetTy {
        let bit = bit.into_u32();
        let idx = (bit >> 6) as usize;
        self.0[idx] |= 1 << (bit & 63);

    }
    

    pub const fn remove_bit(&mut self, bit: Ty) where Ty: [const] BitsetTy {
        let bit = bit.into_u32();
        let idx = (bit >> 6) as usize;
        self.0[idx] &= !(1 << (bit & 63));
    }

    pub const fn contains_bit(&self, bit: Ty) -> bool where Ty: [const] BitsetTy {
        let bit = bit.into_u32();
        let idx = (bit >> 6) as usize;

        (self.0[idx] & (1 << (bit & 63))) != 0
    }

    pub fn retain_mask(&mut self, other: Self) {
        for (a, b) in core::iter::zip(&mut self.0, other.0) {
            *a &= b;
        }
    }

    pub fn remove_mask(&mut self, other: Self) {
        for (a, b) in core::iter::zip(&mut self.0, other.0) {
            *a &= !b;
        }
    }
}

impl<Ty: BitsetTy, const N: usize> FromIterator<Ty> for Bitset<Ty, N> {
    fn from_iter<T: IntoIterator<Item = Ty>>(iter: T) -> Self {
        let mut v = const { Self::new() };
        v.extend(iter);
        v
    }
}

impl<Ty: BitsetTy, const N: usize> Extend<Ty> for Bitset<Ty, N> {
    fn extend<T: IntoIterator<Item = Ty>>(&mut self, iter: T) {
        for bit in iter {
            self.insert_bit(bit);
        }
    }
}

impl<Ty: BitsetTy, const N: usize> IntoIterator for Bitset<Ty, N> {
    type Item = Ty;
    type IntoIter = BitsetIter<Ty, N>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = self.0.into_iter();

        let val = iter.next().unwrap();
        BitsetIter(iter, val, 0, PhantomData)
    }
}

pub struct BitsetIter<Ty, const N: usize>(core::array::IntoIter<u64, N>, u64, u32, PhantomData<[Ty]>);

impl<Ty: BitsetTy, const N: usize> Iterator for BitsetIter<Ty, N> {
    type Item = Ty;

    fn next(&mut self) -> Option<Self::Item> {
        while self.1 == 0 {
            self.2 = (self.2 & 63) + 64;
            self.1 = self.0.next()?;
        }

        let p = self.1.trailing_zeros();
        self.1 >>= p + 1;
        let val = self.2 + p;
        self.2 += p + 1;

        Some(Ty::from_u32(val))
    }
}
