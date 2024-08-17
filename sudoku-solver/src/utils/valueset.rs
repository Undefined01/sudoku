use crate::sudoku::CellValue;

use std::cell::OnceCell;
use std::iter::{Copied, FromIterator};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Sub, SubAssign};
use std::usize;

use arrayvec::ArrayVec;
use bitset_core::BitSet;

#[derive(Debug, Clone)]
pub struct ValueSet {
    bitset: u16,
    values: OnceCell<ArrayVec<CellValue, 9>>,
}

impl ValueSet {
    pub fn new() -> Self {
        ValueSet {
            bitset: 0,
            values: OnceCell::new(),
        }
    }

    pub fn from_bitset(bitset: u16) -> Self {
        ValueSet {
            bitset,
            values: OnceCell::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bitset == 0
    }

    pub fn size(&self) -> usize {
        self.bitset.count_ones() as usize
    }

    pub fn add(&mut self, value: CellValue) {
        self.values.take();
        self.bitset.bit_set(value as usize - 1);
    }

    pub fn has(&self, value: CellValue) -> bool {
        self.bitset.bit_test(value as usize - 1)
    }

    pub fn delete(&mut self, value: CellValue) {
        self.values.take();
        self.bitset.bit_reset(value as usize - 1);
    }

    pub fn clear(&mut self) {
        self.values.take();
        self.bitset = 0;
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.bitset.bit_subset(&other.bitset)
    }

    pub fn union_multiple<'a>(iter: impl Iterator<Item = &'a Self>) -> Self {
        let mut union = Self::new();
        for set in iter {
            union.bitset |= set.bitset;
        }
        union
    }

    pub fn intersection_multiple<'a>(mut iter: impl Iterator<Item = &'a Self>) -> Self {
        let first = iter.next().unwrap();
        let mut intersection = Self::from_bitset(first.bitset);
        for set in iter {
            intersection.bitset &= set.bitset;
        }
        intersection
    }

    pub fn values(&self) -> &[CellValue] {
        self.values.get_or_init(|| {
            let mut values = ArrayVec::new();
            if !self.is_empty() {
                for i in 0..9 {
                    if self.bitset.bit_test(i) {
                        values.push(i as CellValue + 1);
                    }
                }
            }
            values
        })
    }

    pub fn single_value(&self) -> CellValue {
        match self.bitset.trailing_zeros() {
            16 => panic!("ValueSet is empty"),
            idx => idx as CellValue + 1,
        }
    }

    pub fn iter(&self) -> Copied<std::slice::Iter<CellValue>> {
        self.values().iter().copied()
    }
}

impl FromIterator<CellValue> for ValueSet {
    fn from_iter<T: IntoIterator<Item = CellValue>>(iter: T) -> Self {
        let mut set = Self::new();
        let mut array = ArrayVec::new();
        for v in iter {
            if set.has(v) {
                continue;
            }
            array.push(v);
            set.add(v);
        }
        set.values = array.into();
        set
    }
}

impl SubAssign<&ValueSet> for ValueSet {
    fn sub_assign(&mut self, other: &ValueSet) {
        self.values.take();
        self.bitset &= !other.bitset;
    }
}

impl Sub for &ValueSet {
    type Output = ValueSet;

    fn sub(self, other: Self) -> Self::Output {
        ValueSet::from_bitset(self.bitset & !other.bitset)
    }
}

impl BitOrAssign<&ValueSet> for ValueSet {
    fn bitor_assign(&mut self, other: &ValueSet) {
        self.values.take();
        self.bitset |= other.bitset;
    }
}

impl BitOr for &ValueSet {
    type Output = ValueSet;

    fn bitor(self, other: Self) -> Self::Output {
        ValueSet::from_bitset(self.bitset | other.bitset)
    }
}

impl BitAndAssign<&ValueSet> for ValueSet {
    fn bitand_assign(&mut self, other: &ValueSet) {
        self.values.take();
        self.bitset &= other.bitset;
    }
}

impl BitAnd for &ValueSet {
    type Output = ValueSet;

    fn bitand(self, other: Self) -> Self::Output {
        ValueSet::from_bitset(self.bitset & other.bitset)
    }
}

impl PartialEq for ValueSet {
    fn eq(&self, other: &Self) -> bool {
        self.bitset == other.bitset
    }
}

impl Eq for ValueSet {}

impl<'a> IntoIterator for &'a ValueSet {
    type Item = CellValue;
    type IntoIter = Copied<std::slice::Iter<'a, CellValue>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Index<usize> for &ValueSet {
    type Output = CellValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values()[index]
    }
}
