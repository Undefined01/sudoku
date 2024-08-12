use arrayvec::ArrayVec;
use itertools::Itertools;

use super::{CellIndex, Sudoku};

use std::cell::{LazyCell, OnceCell};
use std::iter::{Copied, FromIterator};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut, Sub, SubAssign};


#[derive(Clone)]
pub struct CellSet {
    bitset: u128,
    cells: OnceCell<ArrayVec<CellIndex, 81>>,
}

impl CellSet {
    pub fn new() -> Self {
        CellSet {
            bitset: 0,
            cells: OnceCell::new(),
        }
    }

    pub fn from_bitset(bitset: u128) -> Self {
        CellSet {
            bitset,
            cells: OnceCell::new(),
        }
    }

    pub fn from_cells(cell_positions: Vec<CellIndex>) -> Self {
        let mut set = Self::new();
        for cell in &cell_positions {
            set.add(*cell);
        }
        let array = ArrayVec::from_iter(cell_positions);
        set.cells = array.into();
        set
    }

    pub fn is_empty(&self) -> bool {
        self.bitset == 0
    }

    pub fn size(&self) -> usize {
        self.bitset.count_ones() as usize
    }

    pub fn add(&mut self, cell: CellIndex) {
        self.cells.take();
        self.bitset |= 1 << cell;
    }

    pub fn has(&self, cell: CellIndex) -> bool {
        (self.bitset & (1 << cell)) != 0
    }

    pub fn delete(&mut self, cell: CellIndex) {
        self.cells.take();
        self.bitset &= !(1 << cell);
    }

    pub fn clear(&mut self) {
        self.cells.take();
        self.bitset = 0;
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        (self.bitset & other.bitset) == self.bitset
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

    pub fn iter(&self) -> Copied<std::slice::Iter<CellIndex>> {
        self.cells
            .get_or_init(|| {
                let mut cells = ArrayVec::new();
                if !self.is_empty() {
                    for idx in (0..81).step_by(9) {
                        let bits = ((self.bitset >> idx) & 0x1FF) as u32;
                        if bits == 0 {
                            continue;
                        }
                        for i in 0..9 {
                            if (bits & (1 << i)) != 0 {
                                cells.push(idx + i);
                            }
                        }
                    }
                }
                cells
            })
            .iter()
            .copied()
    }

    pub fn to_string(&self, sudoku: &Sudoku) -> String {
        self.iter().map(|cell| sudoku.get_cell_name(cell)).join(",")
    }
}

impl SubAssign<&CellSet> for CellSet {
    fn sub_assign(&mut self, other: &CellSet) {
        self.cells.take();
        self.bitset &= !other.bitset;
    }
}

impl Sub for &CellSet {
    type Output = CellSet;

    fn sub(self, other: Self) -> Self::Output {
        CellSet::from_bitset(self.bitset & !other.bitset)
    }
}

impl BitOrAssign<&CellSet> for CellSet {
    fn bitor_assign(&mut self, other: &CellSet) {
        self.cells.take();
        self.bitset |= other.bitset;
    }
}

impl BitOr for &CellSet {
    type Output = CellSet;

    fn bitor(self, other: Self) -> Self::Output {
        CellSet::from_bitset(self.bitset | other.bitset)
    }
}

impl BitAndAssign<&CellSet> for CellSet {
    fn bitand_assign(&mut self, other: &CellSet) {
        self.cells.take();
        self.bitset &= other.bitset;
    }
}

impl BitAnd for &CellSet {
    type Output = CellSet;

    fn bitand(self, other: Self) -> Self::Output {
        CellSet::from_bitset(self.bitset & other.bitset)
    }
}

impl PartialEq for CellSet {
    fn eq(&self, other: &Self) -> bool {
        self.bitset == other.bitset
    }
}

impl Eq for CellSet {}

impl<'a> IntoIterator for &'a CellSet {
    type Item = CellIndex;
    type IntoIter = Copied<std::slice::Iter<'a, CellIndex>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct NamedCellSet {
    name: String,
    idx: usize,
    cells: CellSet,
}

impl NamedCellSet {
    pub fn new(name: String, idx: usize) -> Self {
        NamedCellSet {
            name,
            idx,
            cells: CellSet::new(),
        }
    }

    pub fn from_cellset(name: String, cells: CellSet) -> Self {
        NamedCellSet {
            name,
            cells,
            idx: 100,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

impl Deref for NamedCellSet {
    type Target = CellSet;

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

impl DerefMut for NamedCellSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cells
    }
}

impl BitOr for &NamedCellSet {
    type Output = CellSet;

    fn bitor(self, other: Self) -> Self::Output {
        &self.cells | &other.cells
    }
}

impl BitAnd for &NamedCellSet {
    type Output = CellSet;

    fn bitand(self, other: Self) -> Self::Output {
        &self.cells & &other.cells
    }
}

impl BitOr<&CellSet> for &NamedCellSet {
    type Output = CellSet;

    fn bitor(self, other: &CellSet) -> Self::Output {
        &self.cells | other
    }
}

impl BitAnd<&CellSet> for &NamedCellSet {
    type Output = CellSet;

    fn bitand(self, other: &CellSet) -> Self::Output {
        &self.cells & other
    }
}
