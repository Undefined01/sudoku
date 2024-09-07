//! The board of the sudoku puzzle is divided into three bands.
//! Each band is composed of three 3x3 blocks (which is also three rows).
//! Each block is composed of three triads, and each triad is composed of three cells.
//! See the documentation of the `Block`, `TriadsOfBand`, `BandConfigurations`, and `BandConfigurationEliminations` for more details.

use std::array;
use std::intrinsics::{assume, likely};
use std::ops::BitOrAssign;
use std::simd::num::SimdUint;
use std::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdInt,
};
use std::simd::{i16x16, i64x2, simd_swizzle, u16x16, u16x8, u64x1, u64x2, u8x16};
use std::sync::LazyLock;

use itertools::Itertools;

/// The band related data.
///
/// `eliminations` caches the unpropagated eliminations for the configurations in the band.
/// `configurations & !eliminations` is the final configurations for the band.
///
/// The configuration layout is defined for horizontal bands. For vertical bands, the configuration is transposed.
#[derive(Debug, Clone)]
pub struct Band {
    configurations: BandConfigurations,
    eliminations: BandConfigurationEliminations,
}

impl Band {
    pub fn new() -> Self {
        Self {
            configurations: BandConfigurations(u16x8::from_array([
                0b111_111_111,
                0b111_111_111,
                0b111_111_111,
                0b111_111_111,
                0b111_111_111,
                0b111_111_111,
                0,
                0,
            ])),
            eliminations: BandConfigurationEliminations(u16x8::splat(0)),
        }
    }
}

const CONFIGURATION_LAYOUT_STR: [&str; 6] = [
    "
        +-------+-------+-------+
        | X X X | . . . | . . . |
        | . . . | X X X | . . . |
        | . . . | . . . | X X X |
        +-------+-------+-------+
    ",
    "
        +-------+-------+-------+
        | . . . | . . . | X X X |
        | X X X | . . . | . . . |
        | . . . | X X X | . . . |
        +-------+-------+-------+
    ",
    "
        +-------+-------+-------+
        | . . . | X X X | . . . |
        | . . . | . . . | X X X |
        | X X X | . . . | . . . |
        +-------+-------+-------+
    ",
    "
        +-------+-------+-------+
        | . . . | . . . | X X X |
        | . . . | X X X | . . . |
        | X X X | . . . | . . . |
        +-------+-------+-------+
    ",
    "
        +-------+-------+-------+
        | X X X | . . . | . . . |
        | . . . | . . . | X X X |
        | . . . | X X X | . . . |
        +-------+-------+-------+
    ",
    "
        +-------+-------+-------+
        | . . . | X X X | . . . |
        | X X X | . . . | . . . |
        | . . . | . . . | X X X |
        +-------+-------+-------+
    ",
];

/// The layout of the configurations in the band.
/// Stored in row-major order.
/// ```plaintext
/// +----------+----------+----------+
/// |  0  1  2 |  3  4  5 |  6  7  8 |
/// |  9 10 11 | 12 13 14 | 15 16 17 |
/// | 18 19 20 | 21 22 23 | 24 25 26 |
/// +----------+----------+----------+
/// ```
static CONFIGURATION_LAYOUT: LazyLock<[[bool; 27]; 6]> = LazyLock::new(|| {
    let mut mask = [[false; 27]; 6];
    for (configuration, &s) in CONFIGURATION_LAYOUT_STR.iter().enumerate() {
        assert!(s.chars().filter(|&c| c == 'X' || c == '.').count() == 27);
        s.chars()
            .filter(|&c| c == 'X' || c == '.')
            .enumerate()
            .for_each(|(element, c)| mask[configuration][element] = c == 'X');
    }
    mask
});

/// The layout of the configurations in the band.
/// Stored in row-major order.
/// ```plaintext
/// +-------+
/// | 0 1 2 |
/// | 3 4 5 |
/// | 6 7 8 |
/// +-------+
static CONFIGURATION_LAYOUT_FOR_TRIAD: LazyLock<[[bool; 9]; 6]> = LazyLock::new(|| {
    CONFIGURATION_LAYOUT.map(|layout| {
        let mut mask = [false; 9];
        for i in 0..9 {
            mask[i] = layout[i * 3] || layout[i * 3 + 1] || layout[i * 3 + 2];
        }
        mask
    })
});

/// The state of a block in the sudoku.
/// Each u16 integer represents a cell or a triad of cells in the block.
/// The following diagram shows the structure of the block. The last integer in the matrix is unused.
/// ```plaintext
/// +----+----+----+----+
/// | c0 | c1 | c2 | H0 |
/// +----+----+----+----+
/// | c3 | c4 | c5 | H1 |
/// +----+----+----+----+
/// | c6 | c7 | c8 | H2 |
/// +----+----+----+----+
/// | V0 | V1 | V2 | .  |
/// +----+----+----+----+
/// ```
/// The lowest 9 bits in the u16 integer represent the possible values in the cell.
/// H_i holds the NEGATIVE horizontal triads, which means $H0 \leftrightarrow \lnot (c0 \lor c1 \lor c2)$.
/// V_i are the same as H_i, but for the vertical triads.
/// Initially, each cell has all the candidates, and each negative triad also has all the candidates.
/// The invariant is that if the sudoku is valid, each cell has at least the solution as a candidate.
/// If a candidate is eliminated (to zero) from a cell, it cannot be filled in the cell.
/// If a candidate is eliminated (to zero) from a negative triad, it must be filled in the three cells in the triad.
#[derive(Debug, Clone)]
pub struct Block(u16x16);

impl Block {
    pub fn new() -> Self {
        let mut default_value = [0b111_111_111; 16];
        default_value[15] = 0;
        Self(u16x16::from_array(default_value))
    }

    pub fn eliminate(&mut self, mask: &BlockEliminations) -> bool {
        let eliminated = (self.0 & mask.0) != u16x16::splat(0);
        self.0 &= !mask.0;
        eliminated
    }

    pub fn is_subset_of(&self, other: &Block) -> bool {
        (self.0 & other.0) == self.0
    }

    pub fn simd_count_ones(&self) -> u16x16 {
        self::simd_count_ones(&self.0)
    }
}

impl BitOrAssign<&Block> for Block {
    fn bitor_assign(&mut self, rhs: &Block) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone)]
pub struct BlockEliminations(u16x16);

struct BlockIndex {
    block_r: u8,
    block_c: u8,
    block_idx: u8,
    element_r: u8,
    element_c: u8,
    /// The index of the element in the block
    element_idx: u8,
}

impl BlockIndex {
    #[inline(always)]
    pub fn from_cell(cell: u8) -> BlockIndex {
        let block_r = cell / 27;
        let block_c = (cell % 9) / 3;
        let block_idx = block_r * 3 + block_c;
        let element_r = (cell / 9) % 3;
        let element_c = cell % 3;
        let element_idx = element_r * 4 + element_c;
        BlockIndex {
            block_r,
            block_c,
            block_idx,
            element_r,
            element_c,
            element_idx,
        }
    }

    #[inline(always)]
    pub fn transpose(block_index: &BlockIndex) -> BlockIndex {
        BlockIndex {
            block_r: block_index.block_c,
            block_c: block_index.block_r,
            block_idx: block_index.block_c * 3 + block_index.block_r,
            element_r: block_index.element_c,
            element_c: block_index.element_r,
            element_idx: block_index.element_c * 4 + block_index.element_r,
        }
    }
}

///
/// For a specific value, there are exactly one triad that contains the value in each block and each row.
/// Hence, there are only 6 possible configurations for how that value can be placed in the triads of a band.
/// Hence, we can use 6 integers to represent which triads can contain the value in the band.
/// Different from the triads in the block, the configuration holds the positive triads.
#[derive(Debug, Clone)]
pub struct BandConfigurations(u16x8);

impl BandConfigurations {
    #[inline(always)]
    pub fn eliminate(&mut self, mask: &BandConfigurationEliminations) -> bool {
        let eliminated = (self.0 & mask.0) != u16x8::splat(0);
        self.0 &= !mask.0;
        eliminated
    }

    /// Convert the configurations of the band to the triads of the band.
    /// This equals to shuffling and doing bitor by the following pattern (dot means the bit is not used and is zero):
    /// ```plaintext
    /// c0|c4 c1|c5 c2|c3 .
    /// c2|c5 c0|c3 c1|c4 .
    /// c1|c3 c2|c4 c0|c5 .
    /// .     .     .     .
    /// ```
    pub fn to_triads(&self) -> TriadsOfBand {
        let possibility1 = simd_swizzle!(self.0, BAND_CONFIGURATION_TO_TRIADS[0]);
        let possibility2 = simd_swizzle!(self.0, BAND_CONFIGURATION_TO_TRIADS[1]);
        TriadsOfBand(possibility1 | possibility2)
    }
}

const BAND_CONFIGURATION_TO_TRIADS: [[usize; 16]; 2] = [
    [0, 1, 2, 7, 2, 0, 1, 7, 1, 2, 0, 7, 7, 7, 7, 7],
    [4, 5, 3, 7, 5, 3, 4, 7, 3, 4, 5, 7, 7, 7, 7, 7],
];

#[cfg(test)]
mod configuration_test {
    use super::*;

    pub fn to_triads_naive(configurations: &[u16; 8]) -> TriadsOfBand {
        let mut triads = [0u16; 16];
        let mut triads_mask = array::from_fn::<_, 16, _>(|_| Vec::new());
        for triad_r in 0..3 {
            for triad_c in 0..3 {
                let mut triad = 0;
                for configuration in 0..6 {
                    if CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_r * 3 + triad_c] {
                        triads[triad_r + triad_c * 4] |= configurations[configuration];
                        triads_mask[triad_r + triad_c * 4].push(configuration);
                    }
                }
            }
        }
        if cfg!(debug_assertions) {
            for i in (0..16).step_by(4) {
                for j in i..i + 4 {
                    if triads_mask[j].len() >= 1 {
                        debug_assert_eq!(triads_mask[j].len(), 2);
                        eprint!(
                            "{} ",
                            triads_mask[j].iter().map(|x| x.to_string()).join("|")
                        );
                    } else {
                        eprint!(".   ");
                    }
                }
                eprintln!();
            }
            let possibility1 = triads_mask
                .iter()
                .map(|m| if m.len() == 2 { m[0] } else { 7 })
                .collect::<Vec<_>>();
            let possibility2 = triads_mask
                .iter()
                .map(|m| if m.len() == 2 { m[1] } else { 7 })
                .collect::<Vec<_>>();
            debug_assert_eq!(possibility1, BAND_CONFIGURATION_TO_TRIADS[0]);
            debug_assert_eq!(possibility2, BAND_CONFIGURATION_TO_TRIADS[1]);
        }
        TriadsOfBand(u16x16::from_array(triads))
    }

    #[test]
    fn test_to_traids() {
        let configurations = [
            0b111000111,
            0b000011110,
            0b110011001,
            0b001100111,
            0b101011001,
            0b010100111,
            0,
            0,
        ];
        let band_configurations = BandConfigurations(u16x8::from_slice(&configurations));
        assert_eq!(
            band_configurations.to_triads(),
            to_triads_naive(&configurations)
        );
    }
}

/// There are nine horizontal triad in each band, and each triad is composed of three cells in the intersection of a row and a block.
/// ```plaintext
/// +-------+-------+-------+
/// | 0 0 0 | 3 3 3 | 6 6 6 |
/// | 1 1 1 | 4 4 4 | 7 7 7 |
/// | 2 2 2 | 5 5 5 | 8 8 8 |
/// +-------+-------+-------+
/// ```
/// The nine horizontal triad in a band can be represented as a 3x3 matrix of triads.
/// Each u16 integer represents a triad of cells in the band, and dot means the bit is not used and is zero.
/// The lowest 9 bits in the u16 integer represent the possible values in the triad.
/// ```plaintext
/// +---------+
/// | 0 1 2 . |
/// | 3 4 5 . |
/// | 6 7 8 . |
/// | . . . . |
/// +---------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriadsOfBand(u16x16);

impl TriadsOfBand {
    pub fn simd_count_ones(&self) -> u16x16 {
        self::simd_count_ones(&self.0)
    }

    pub fn to_candidates_in_block(&self, is_vertial_triad: bool) -> [Block; 3] {
        let ones_filled_triads = self.0
            | u16x16::from_array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b111_111_111]);
        if !is_vertial_triad {
            // Note that the triads in the block are negative triads.
            // By the constraint that for each candidate, there are at most one triad that is true in a block, we have H0 = ~H1 & ~H2, H1 = ~H0 & ~H2, H2 = ~H0 & ~H1.
            // Hence, we can shuffle the triads to get the candidates mask for each block (star means the bit is not used and is ones):
            // +-----------+
            // | 0 0 0 1|2 |
            // | 1 1 1 0|2 |
            // | 2 2 2 0|1 |
            // | * * *  *  |
            // +-----------+
            let r1 = simd_swizzle!(
                ones_filled_triads,
                [0, 0, 0, 1, 1, 1, 1, 0, 2, 2, 2, 0, 15, 15, 15, 15]
            ) | simd_swizzle!(
                ones_filled_triads,
                [0, 0, 0, 2, 1, 1, 1, 2, 2, 2, 2, 1, 15, 15, 15, 15]
            );
            let r2 = simd_swizzle!(
                ones_filled_triads,
                [4, 4, 4, 5, 5, 5, 5, 4, 6, 6, 6, 4, 15, 15, 15, 15]
            ) | simd_swizzle!(
                ones_filled_triads,
                [4, 4, 4, 6, 5, 5, 5, 6, 6, 6, 6, 5, 15, 15, 15, 15]
            );
            let r3 = simd_swizzle!(
                ones_filled_triads,
                [8, 8, 8, 9, 9, 9, 9, 8, 10, 10, 10, 8, 15, 15, 15, 15]
            ) | simd_swizzle!(
                ones_filled_triads,
                [8, 8, 8, 10, 9, 9, 9, 10, 10, 10, 10, 9, 15, 15, 15, 15]
            );
            return [Block(r1), Block(r2), Block(r3)];
        } else {
            // +---------------+
            // |  0   1   2  * |
            // |  0   1   2  * |
            // |  0   1   2  * |
            // | 1|2 0|2 0|1 * |
            // +---------------+
            let r1 = simd_swizzle!(
                ones_filled_triads,
                [0, 1, 2, 15, 0, 1, 2, 15, 0, 1, 2, 15, 1, 0, 0, 15]
            ) | simd_swizzle!(
                ones_filled_triads,
                [0, 1, 2, 15, 0, 1, 2, 15, 0, 1, 2, 15, 2, 2, 1, 15]
            );
            let r2 = simd_swizzle!(
                ones_filled_triads,
                [4, 5, 6, 15, 4, 5, 6, 15, 4, 5, 6, 15, 5, 4, 4, 15]
            ) | simd_swizzle!(
                ones_filled_triads,
                [4, 5, 6, 15, 4, 5, 6, 15, 4, 5, 6, 15, 6, 6, 5, 15]
            );
            let r3 = simd_swizzle!(
                ones_filled_triads,
                [8, 9, 10, 15, 8, 9, 10, 15, 8, 9, 10, 15, 9, 8, 8, 15]
            ) | simd_swizzle!(
                ones_filled_triads,
                [8, 9, 10, 15, 8, 9, 10, 15, 8, 9, 10, 15, 10, 10, 9, 15]
            );
            return [Block(r1), Block(r2), Block(r3)];
        }
    }
}

pub fn simd_count_ones(v: &u16x16) -> u16x16 {
    // Currently (2024-09-04), the `std::simd::Simd` does not have the `count_ones` method.
    // So we use the `std::intrinsics::simd::simd_ctpop` instead.
    unsafe { std::intrinsics::simd::simd_ctpop(*v) }
}

#[cfg(test)]
mod count_ones_test {
    use super::*;

    pub fn count_ones_naive(v: &u16x16) -> u16x16 {
        let mut counts = u16x16::splat(0);
        for i in 0..16 {
            counts += (v >> i) & u16x16::splat(1);
        }
        counts
    }

    #[test]
    fn test_count_ones() {
        let triads = u16x16::splat(0b1010101010101010);
        assert_eq!(simd_count_ones(&triads), count_ones_naive(&triads),);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandConfigurationEliminations(u16x8);

/// Eliminate the configurations that do not contain the asserted triad
/// configuration 0 is eliminated if triad 1, 2, 3, 5, 6, 7 is asserted
/// configuration 1 is eliminated if triad 0, 2, 3, 4, 7, 8 is asserted
/// configuration 2 is eliminated if triad 0, 1, 4, 5, 6, 8 is asserted
/// configuration 3 is eliminated if triad 0, 1, 3, 5, 7, 8 is asserted
/// configuration 4 is eliminated if triad 1, 2, 3, 4, 6, 8 is asserted
/// configuration 5 is eliminated if triad 0, 2, 4, 5, 6, 7 is asserted
/// We can shuffle the asserting bitvector to construct the elimination mask
/// See the `test_elimination_from_triad` test for the correctness of the elimination mask
const TRIAD_ASSERTION_TO_ELIMINATION: [[usize; 8]; 6] = [
    [1, 0, 0, 0, 1, 0, 7, 7],
    [2, 2, 1, 1, 2, 2, 7, 7],
    [0, 0, 1, 0, 0, 1, 7, 7],
    [2, 1, 2, 2, 1, 2, 7, 7],
    [0, 1, 0, 1, 0, 0, 7, 7],
    [1, 2, 2, 2, 2, 1, 7, 7],
];

impl BandConfigurationEliminations {
    pub fn from_triad(triad_assertion: &TriadsOfBand) -> BandConfigurationEliminations {
        let assertion_r1 = u16x8::from_slice(&triad_assertion.0.as_array()[..8]);
        let assertion_r2 = u16x8::from_slice(&triad_assertion.0.as_array()[4..12]);
        let assertion_r3 = u16x8::from_slice(&triad_assertion.0.as_array()[8..16]);
        let mut eliminations = simd_swizzle!(assertion_r1, TRIAD_ASSERTION_TO_ELIMINATION[0]);
        eliminations |= simd_swizzle!(assertion_r1, TRIAD_ASSERTION_TO_ELIMINATION[1]);
        eliminations |= simd_swizzle!(assertion_r2, TRIAD_ASSERTION_TO_ELIMINATION[2]);
        eliminations |= simd_swizzle!(assertion_r2, TRIAD_ASSERTION_TO_ELIMINATION[3]);
        eliminations |= simd_swizzle!(assertion_r3, TRIAD_ASSERTION_TO_ELIMINATION[4]);
        eliminations |= simd_swizzle!(assertion_r3, TRIAD_ASSERTION_TO_ELIMINATION[5]);
        BandConfigurationEliminations(eliminations)
    }

    #[inline(always)]
    pub fn from_asserted_negative_triad(
        block_idx: usize,
        asserted_negative_triads: &Block,
    ) -> u16x16 {
        // Currently (2024-09-04), the `std::simd::Simd` does not fully support swizzling with a dynamic index.
        // So we jump to the correct swizzle by a match expression.
        match block_idx {
            0 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[0]
            ),
            1 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[1]
            ),
            2 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[2]
            ),
            3 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[3]
            ),
            4 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[4]
            ),
            5 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[5]
            ),
            6 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[6]
            ),
            7 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[7]
            ),
            8 => simd_swizzle!(
                asserted_negative_triads.0,
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[8]
            ),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn from_eliminated_negative_triad(
        block_idx: usize,
        eliminated_negative_triads: &BlockEliminations,
    ) -> u16x16 {
        // Currently (2024-09-04), the `std::simd::Simd` does not fully support swizzling with a dynamic index.
        // So we jump to the correct swizzle by a match expression.
        match block_idx {
            0 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[0][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[0][1]
                )
            }
            1 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[1][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[1][1]
                )
            }
            2 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[2][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[2][1]
                )
            }
            3 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[3][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[3][1]
                )
            }
            4 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[4][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[4][1]
                )
            }
            5 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[5][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[5][1]
                )
            }
            6 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[6][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[6][1]
                )
            }
            7 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[7][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[7][1]
                )
            }
            8 => {
                simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[8][0]
                ) | simd_swizzle!(
                    eliminated_negative_triads.0,
                    ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[8][1]
                )
            }
            _ => unreachable!(),
        }
    }
}

impl BitOrAssign<&BandConfigurationEliminations> for BandConfigurationEliminations {
    fn bitor_assign(&mut self, rhs: &BandConfigurationEliminations) {
        self.0 |= rhs.0;
    }
}

#[cfg(test)]
mod band_configuration_eliminations_test {
    use super::*;

    pub fn elimination_from_triad_naive(triad_assertion: &u16x16) -> BandConfigurationEliminations {
        let assertion = triad_assertion.to_array();
        let mut eliminations = [0u16; 8];
        let mut eliminations_mask: [_; 6] = array::from_fn(|_| vec![]);
        for configuration in 0..6 {
            for triad_r in 0..3 {
                for triad_c in 0..3 {
                    // Eliminate the configuration that does not contain the asserted triad
                    if !CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_r * 3 + triad_c] {
                        eliminations[configuration] |= assertion[triad_r + triad_c * 4];
                        eliminations_mask[configuration].push(triad_r + triad_c * 4);
                    }
                }
            }
            eliminations_mask[configuration].sort();
        }
        if cfg!(debug_assertions) {
            for i in 0..6 {
                eprintln!(
                    "configuration {} is eliminated if triad {} is asserted",
                    i,
                    eliminations_mask[i].iter().join(", ")
                );
            }
            #[rustfmt::skip]
            (|| {
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[0]).collect::<Vec<_>>(), [1, 0, 0, 0, 1, 0]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[1]).collect::<Vec<_>>(), [2, 2, 1, 1, 2, 2]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[2]).collect::<Vec<_>>(), [4, 4, 5, 4, 4, 5]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[3]).collect::<Vec<_>>(), [6, 5, 6, 6, 5, 6]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[4]).collect::<Vec<_>>(), [8, 9, 8, 9, 8, 8]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[5]).collect::<Vec<_>>(), [9, 10, 10, 10, 10, 9]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[0] - 0).chain([7, 7].into_iter()).collect::<Vec<_>>(), TRIAD_ASSERTION_TO_ELIMINATION[0]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[1] - 0).chain([7, 7].into_iter()).collect::<Vec<_>>(), TRIAD_ASSERTION_TO_ELIMINATION[1]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[2] - 4).chain([7, 7].into_iter()).collect::<Vec<_>>(), TRIAD_ASSERTION_TO_ELIMINATION[2]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[3] - 4).chain([7, 7].into_iter()).collect::<Vec<_>>(), TRIAD_ASSERTION_TO_ELIMINATION[3]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[4] - 8).chain([7, 7].into_iter()).collect::<Vec<_>>(), TRIAD_ASSERTION_TO_ELIMINATION[4]);
                debug_assert_eq!(eliminations_mask.iter().map(|m| m[5] - 8).chain([7, 7].into_iter()).collect::<Vec<_>>(), TRIAD_ASSERTION_TO_ELIMINATION[5]);
            })();
        }
        BandConfigurationEliminations(u16x8::from_array(eliminations))
    }

    #[test]
    fn test_elimination_from_triad() {
        #[rustfmt::skip]
        let triad = u16x16::from_array([
            0b101_000_010, 0b010_101_000, 0b000_000_000, 0b000_000_000,
            0b000_101_001, 0b000_000_000, 0b111_000_000, 0b000_000_000,
            0b010_010_100, 0b000_000_000, 0b000_101_010, 0b000_000_000,
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_000,
        ]);
        assert_eq!(
            BandConfigurationEliminations::from_triad(&TriadsOfBand(triad)),
            elimination_from_triad_naive(&triad),
        );
    }

    fn from_asserted_negative_triad(
        block_idx: usize,
        eliminated_triads: &Block,
    ) -> (BandConfigurationEliminations, BandConfigurationEliminations) {
        let row_idx = block_idx / 3;
        let column_idx = block_idx % 3;
        let mut horizontal_eliminations = [0u16; 8];
        let mut horizontal_eliminations_mask: [_; 6] = array::from_fn(|_| vec![]);
        let mut vertical_eliminations = [0u16; 8];
        let mut vertical_eliminations_mask: [_; 6] = array::from_fn(|_| vec![]);
        for configuration in 0..6 {
            for triad_idx in 0..3 {
                let horizontal_triad_idx_in_block = triad_idx * 4 + 3;
                let triad_idx_in_band = triad_idx * 3 + column_idx;
                // Eliminate the configuration that contains the eliminated triad
                if CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_idx_in_band] {
                    horizontal_eliminations[configuration] |=
                        eliminated_triads.0[horizontal_triad_idx_in_block];
                    horizontal_eliminations_mask[configuration].push(horizontal_triad_idx_in_block);
                }
                let vertical_triad_idx_in_block = 3 * 4 + triad_idx;
                let triad_idx_in_band = triad_idx * 3 + row_idx;
                // Eliminate the configuration that contains the eliminated triad
                if CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_idx_in_band] {
                    vertical_eliminations[configuration] |=
                        eliminated_triads.0[vertical_triad_idx_in_block];
                    vertical_eliminations_mask[configuration].push(vertical_triad_idx_in_block);
                }
            }
        }

        if cfg!(debug_assertions) {
            for i in 0..6 {
                // eprintln!(
                //     "configuration {} in horizontal band is eliminated if {} element in the block is eliminated",
                //     i,
                //     horizontal_eliminations_mask[i].iter().join(", ")
                // );
                debug_assert_eq!(horizontal_eliminations_mask[i].len(), 1);
                // eprintln!(
                //     "configuration {} in vertical band is eliminated if {} element in the block is eliminated",
                //     i,
                //     vertical_eliminations_mask[i].iter().join(", ")
                // );
                debug_assert_eq!(vertical_eliminations_mask[i].len(), 1);
            }
            debug_assert_eq!(
                horizontal_eliminations_mask
                    .iter()
                    .map(|m| m[0])
                    .chain([15, 15].into_iter())
                    .chain(vertical_eliminations_mask.iter().map(|m| m[0]))
                    .chain([15, 15].into_iter())
                    .collect::<Vec<_>>(),
                BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[block_idx],
            );
        }

        (
            BandConfigurationEliminations(u16x8::from_array(horizontal_eliminations)),
            BandConfigurationEliminations(u16x8::from_array(vertical_eliminations)),
        )
    }

    pub fn from_eliminated_negative_triad(
        block_idx: usize,
        eliminated_negative_triads: &BlockEliminations,
    ) -> (BandConfigurationEliminations, BandConfigurationEliminations) {
        let row_idx = block_idx / 3;
        let column_idx = block_idx % 3;
        let mut horizontal_eliminations = [0u16; 8];
        let mut horizontal_eliminations_mask: [_; 6] = array::from_fn(|_| vec![]);
        let mut vertical_eliminations = [0u16; 8];
        let mut vertical_eliminations_mask: [_; 6] = array::from_fn(|_| vec![]);
        for configuration in 0..6 {
            for triad_idx in 0..3 {
                let horizontal_triad_idx_in_block = triad_idx * 4 + 3;
                let triad_idx_in_band = triad_idx * 3 + column_idx;
                // Eliminate the configuration that does not contain the triad
                if !CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_idx_in_band] {
                    horizontal_eliminations[configuration] |=
                        eliminated_negative_triads.0[horizontal_triad_idx_in_block];
                    horizontal_eliminations_mask[configuration].push(horizontal_triad_idx_in_block);
                }
                let vertical_triad_idx_in_block = 3 * 4 + triad_idx;
                let triad_idx_in_band = triad_idx * 3 + row_idx;
                // Eliminate the configuration that does not contain the triad
                if !CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_idx_in_band] {
                    vertical_eliminations[configuration] |=
                        eliminated_negative_triads.0[vertical_triad_idx_in_block];
                    vertical_eliminations_mask[configuration].push(vertical_triad_idx_in_block);
                }
            }
        }

        if cfg!(debug_assertions) {
            for i in 0..6 {
                eprintln!(
                    "configuration {} in horizontal band is eliminated if {} element in the block is eliminated",
                    i,
                    horizontal_eliminations_mask[i].iter().join(", ")
                );
                debug_assert_eq!(horizontal_eliminations_mask[i].len(), 2);
                eprintln!(
                    "configuration {} in vertical band is eliminated if {} element in the block is eliminated",
                    i,
                    vertical_eliminations_mask[i].iter().join(", ")
                );
                debug_assert_eq!(vertical_eliminations_mask[i].len(), 2);
            }
            debug_assert_eq!(
                horizontal_eliminations_mask
                    .iter()
                    .map(|m| m[0])
                    .chain([15, 15].into_iter())
                    .chain(vertical_eliminations_mask.iter().map(|m| m[0]))
                    .chain([15, 15].into_iter())
                    .collect::<Vec<_>>(),
                ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[block_idx][0],
            );
            debug_assert_eq!(
                horizontal_eliminations_mask
                    .iter()
                    .map(|m| m[1])
                    .chain([15, 15].into_iter())
                    .chain(vertical_eliminations_mask.iter().map(|m| m[1]))
                    .chain([15, 15].into_iter())
                    .collect::<Vec<_>>(),
                ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION[block_idx][1],
            );
        }

        (
            BandConfigurationEliminations(u16x8::from_array(horizontal_eliminations)),
            BandConfigurationEliminations(u16x8::from_array(vertical_eliminations)),
        )
    }

    #[test]
    fn test_elimination_from_asserted_block_triad() {
        #[rustfmt::skip]
        let block_triad = u16x16::from_array([
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_110,
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_010,
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_011,
            0b000_101_010, 0b110_000_000, 0b010_000_011, 0b000_000_000,
        ]);
        let block_triad = Block(block_triad);

        for block_idx in 0..9 {
            from_asserted_negative_triad(block_idx, &block_triad);
        }
    }

    #[test]
    fn test_elimination_from_eliminated_block_triad() {
        #[rustfmt::skip]
        let block_triad = u16x16::from_array([
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_110,
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_010,
            0b000_000_000, 0b000_000_000, 0b000_000_000, 0b000_000_011,
            0b000_101_010, 0b110_000_000, 0b010_000_011, 0b000_000_000,
        ]);
        let block_triad = BlockEliminations(block_triad);

        for block_idx in 0..9 {
            from_eliminated_negative_triad(block_idx, &block_triad);
        }
    }
}

const BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION: [[usize; 16]; 9] = [
    [3, 7, 11, 11, 3, 7, 15, 15, 12, 13, 14, 14, 12, 13, 15, 15],
    [7, 11, 3, 7, 11, 3, 15, 15, 12, 13, 14, 14, 12, 13, 15, 15],
    [11, 3, 7, 3, 7, 11, 15, 15, 12, 13, 14, 14, 12, 13, 15, 15],
    [3, 7, 11, 11, 3, 7, 15, 15, 13, 14, 12, 13, 14, 12, 15, 15],
    [7, 11, 3, 7, 11, 3, 15, 15, 13, 14, 12, 13, 14, 12, 15, 15],
    [11, 3, 7, 3, 7, 11, 15, 15, 13, 14, 12, 13, 14, 12, 15, 15],
    [3, 7, 11, 11, 3, 7, 15, 15, 14, 12, 13, 12, 13, 14, 15, 15],
    [7, 11, 3, 7, 11, 3, 15, 15, 14, 12, 13, 12, 13, 14, 15, 15],
    [11, 3, 7, 3, 7, 11, 15, 15, 14, 12, 13, 12, 13, 14, 15, 15],
];

const ELIMINATED_BLOCK_TRIAD_TO_CONFIGURATION_ELIMINATION: [[[usize; 16]; 2]; 9] = [
    [
        [7, 3, 3, 3, 7, 3, 15, 15, 13, 12, 12, 12, 13, 12, 15, 15],
        [11, 11, 7, 7, 11, 11, 15, 15, 14, 14, 13, 13, 14, 14, 15, 15],
    ],
    [
        [3, 3, 7, 3, 3, 7, 15, 15, 13, 12, 12, 12, 13, 12, 15, 15],
        [11, 7, 11, 11, 7, 11, 15, 15, 14, 14, 13, 13, 14, 14, 15, 15],
    ],
    [
        [3, 7, 3, 7, 3, 3, 15, 15, 13, 12, 12, 12, 13, 12, 15, 15],
        [7, 11, 11, 11, 11, 7, 15, 15, 14, 14, 13, 13, 14, 14, 15, 15],
    ],
    [
        [7, 3, 3, 3, 7, 3, 15, 15, 12, 12, 13, 12, 12, 13, 15, 15],
        [11, 11, 7, 7, 11, 11, 15, 15, 14, 13, 14, 14, 13, 14, 15, 15],
    ],
    [
        [3, 3, 7, 3, 3, 7, 15, 15, 12, 12, 13, 12, 12, 13, 15, 15],
        [11, 7, 11, 11, 7, 11, 15, 15, 14, 13, 14, 14, 13, 14, 15, 15],
    ],
    [
        [3, 7, 3, 7, 3, 3, 15, 15, 12, 12, 13, 12, 12, 13, 15, 15],
        [7, 11, 11, 11, 11, 7, 15, 15, 14, 13, 14, 14, 13, 14, 15, 15],
    ],
    [
        [7, 3, 3, 3, 7, 3, 15, 15, 12, 13, 12, 13, 12, 12, 15, 15],
        [11, 11, 7, 7, 11, 11, 15, 15, 13, 14, 14, 14, 14, 13, 15, 15],
    ],
    [
        [3, 3, 7, 3, 3, 7, 15, 15, 12, 13, 12, 13, 12, 12, 15, 15],
        [11, 7, 11, 11, 7, 11, 15, 15, 13, 14, 14, 14, 14, 13, 15, 15],
    ],
    [
        [3, 7, 3, 7, 3, 3, 15, 15, 12, 13, 12, 13, 12, 12, 15, 15],
        [7, 11, 11, 11, 11, 7, 15, 15, 13, 14, 14, 14, 14, 13, 15, 15],
    ],
];

pub struct Cache {
    cell_eliminations_in_block: [[BlockEliminations; 16]; 9],
    triad_idx_to_configuration_eliminations: [[BandConfigurationEliminations; 9]; 9],
}

impl Cache {
    /// The eliminations of asserting a value in a cell within a block.
    pub fn cell_eliminations_in_block(&self, value: u8, block: u8) -> &BlockEliminations {
        &self.cell_eliminations_in_block[value as usize][block as usize]
    }

    /// The eliminations of asserting a value in a triad within a band.
    pub fn triad_idx_to_configuration_eliminations(
        &self,
        value: u8,
        element_idx: u8,
    ) -> &BandConfigurationEliminations {
        &self.triad_idx_to_configuration_eliminations[value as usize][element_idx as usize]
    }
}

static CACHE: LazyLock<Cache> = LazyLock::new(|| {
    // Asserting a value in a cell
    let mut cell_eliminations_in_block =
        array::from_fn(|_| array::from_fn(|_| BlockEliminations(u16x16::splat(0))));
    for r in 0..3 {
        for c in 0..3 {
            let cell = r * 4 + c;
            for value in 0..9 {
                let mut eliminations = [0u16; 16];
                // Eliminate the same value from all the other cells in the block
                for i in 0..3 {
                    for j in 0..3 {
                        let other_cell = i * 4 + j;
                        eliminations[other_cell] = 1 << value;
                    }
                }

                // Asserting the value in the cell eliminates all the other candidates in the cell
                eliminations[cell] = !(1 << value);

                // Asserting the value eliminates the negation of triads in the same row and column
                eliminations[r * 4 + 3] = 1 << value;
                eliminations[3 * 4 + c] = 1 << value;

                cell_eliminations_in_block[value as usize][cell] =
                    BlockEliminations(u16x16::from_array(eliminations));
            }
        }
    }

    let mut triad_idx_to_configuration_eliminations =
        array::from_fn(|_| array::from_fn(|_| BandConfigurationEliminations(u16x8::splat(0))));
    for value in 0..9 {
        // Asserting the value in the triad eliminates the configurations that do not contain the value in the triad
        for triad_r in 0..3 {
            for triad_c in 0..3 {
                let mut eliminations = [0u16; 8];
                for configuration in 0..6 {
                    if !CONFIGURATION_LAYOUT_FOR_TRIAD[configuration][triad_r * 3 + triad_c] {
                        eliminations[configuration] = 1 << value;
                    }
                }
                triad_idx_to_configuration_eliminations[value as usize][triad_r + triad_c * 3] =
                    BandConfigurationEliminations(u16x8::from_array(eliminations));
            }
        }
    }

    Cache {
        cell_eliminations_in_block,
        triad_idx_to_configuration_eliminations,
    }
});

/// The state of the sudoku board.
///
/// There are two different representations of the state: the blocks and the bands.
///
/// ## Blocks
/// Blocks are the 3x3 squares in the board.
/// There are nine blocks in a board.
/// ```plaintext
/// +-------+-------+-------+
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// +-------+-------+-------+
/// | 3 3 3 | 4 4 4 | 5 5 5 |
/// | 3 3 3 | 4 4 4 | 5 5 5 |
/// | 3 3 3 | 4 4 4 | 5 5 5 |
/// +-------+-------+-------+
/// | 6 6 6 | 7 7 7 | 8 8 8 |
/// | 6 6 6 | 7 7 7 | 8 8 8 |
/// | 6 6 6 | 7 7 7 | 8 8 8 |
/// +-------+-------+-------+
/// ```
/// To speed up the computation, some extra information is stored in the block. See the `Block` struct for more information.
///
/// ## Bands
/// Bands are composed of three blocks in a row or a column.
/// There are three horizontal bands and three vertical bands in a board.
/// The following diagram shows the structure of the horizontal bands in the board.
/// ```plaintext
/// +-------+-------+-------+
/// | 0 0 0 | 0 0 0 | 0 0 0 |
/// | 0 0 0 | 0 0 0 | 0 0 0 |
/// | 0 0 0 | 0 0 0 | 0 0 0 |
/// +-------+-------+-------+
/// | 1 1 1 | 1 1 1 | 1 1 1 |
/// | 1 1 1 | 1 1 1 | 1 1 1 |
/// | 1 1 1 | 1 1 1 | 1 1 1 |
/// +-------+-------+-------+
/// | 2 2 2 | 2 2 2 | 2 2 2 |
/// | 2 2 2 | 2 2 2 | 2 2 2 |
/// | 2 2 2 | 2 2 2 | 2 2 2 |
/// +-------+-------+-------+
/// ```
/// And the following diagram shows the structure of the vertical bands in the board.
/// ```plaintext
/// +-------+-------+-------+
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// +-------+-------+-------+
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// +-------+-------+-------+
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// | 0 0 0 | 1 1 1 | 2 2 2 |
/// +-------+-------+-------+
/// ```
#[derive(Clone)]
pub struct State {
    /// The bands of the sudoku.
    /// The first dimension is the orientation of the band (0 for horizontal, 1 for vertical).
    /// The second dimension is the index of the band (3 bands in a board).
    bands: [[Band; 3]; 2],
    blocks: [Block; 9],
}

const MINIMUM_COUNT_OF_CANDIDATES_IN_BLOCK: u16x16 =
    u16x16::from_array([1, 1, 1, 6, 1, 1, 1, 6, 1, 1, 1, 6, 6, 6, 6, 0]);

impl State {
    pub fn new() -> Self {
        Self {
            bands: array::from_fn(|_| array::from_fn(|_| Band::new())),
            blocks: array::from_fn(|_| Block::new()),
        }
    }

    pub fn from_values(values: &str) -> Self {
        let mut state = Self::new();
        for (pos, c) in values.chars().enumerate() {
            if c != '.' && c != '0' {
                state.fill(pos as u8, c.to_digit(10).unwrap() as u8 - 1);
            }
        }

        state.band_elimination(false, 0, 1);
        state.band_elimination(true, 0, 1);
        state.band_elimination(false, 1, 2);
        state.band_elimination(true, 1, 2);
        state.band_elimination(false, 2, 0);
        state.band_elimination(true, 2, 0);

        state
    }

    fn fill(&mut self, pos: u8, value: u8) {
        let index = BlockIndex::from_cell(pos);
        self.blocks[index.block_idx as usize]
            .eliminate(CACHE.cell_eliminations_in_block(value, index.element_idx));
        // Note that the triads are column-major
        self.bands[0][index.block_r as usize].eliminations |= CACHE
            .triad_idx_to_configuration_eliminations(value, index.block_c * 3 + index.element_r);
        // In vertical band, the row and column are swapped
        self.bands[1][index.block_c as usize].eliminations |= CACHE
            .triad_idx_to_configuration_eliminations(value, index.block_r * 3 + index.element_c);
    }

    fn band_elimination(
        &mut self,
        is_vertical: bool,
        band_idx: usize,
        from_peer: usize,
    ) -> Result<(), ()> {
        let ref mut band = self.bands[is_vertical as usize][band_idx];
        if likely(!band.configurations.eliminate(&band.eliminations)) {
            return Ok(());
        }
        // println!(
        //     "band_elimination {} {} {}: {}",
        //     is_vertical as usize,
        //     band_idx,
        //     from_peer,
        //     band.configurations
        //         .0
        //         .as_array()
        //         .map(|x| format!("{:>3}", format!("{:o}", x)))
        //         .join(" ")
        // );

        let triads = band.configurations.to_triads();
        let counts = triads.simd_count_ones();

        // If there are less than three candidates to fill a triad, the band is invalid.
        // We skip this check because this is unlikely to happen and the check is expensive.
        // block_restrict will handle the invalid state.
        // if !counts.simd_ge(u16x16::splat(3)).all() {
        //     return Err(());
        // }

        // For each triad, if there only remains three candidates to fill the three cells in the triad, we can assert these three candidates are in the triad, and eliminate the other configurations.
        // This is also known as the "hidden triple" solving technique.
        let asserting_mask: u16x16 = counts.simd_eq(u16x16::splat(3)).to_int().cast();
        let asserting = TriadsOfBand(triads.0 & asserting_mask);
        let elimination = BandConfigurationEliminations::from_triad(&asserting);
        band.configurations.eliminate(&elimination);

        let triads = band.configurations.to_triads();
        let block_masks_in_band = triads.to_candidates_in_block(is_vertical);
        unsafe { assume(band_idx < 3) };
        unsafe { assume(from_peer < 3) };
        let blocks_in_band = if !is_vertical {
            [band_idx * 3, band_idx * 3 + 1, band_idx * 3 + 2]
        } else {
            [band_idx, band_idx + 3, band_idx + 6]
        };
        let peer_idx = [(from_peer + 1) % 3, (from_peer + 2) % 3, from_peer];
        self.block_restrict(
            is_vertical,
            blocks_in_band[peer_idx[0]],
            &block_masks_in_band[peer_idx[0]],
        )?;
        self.block_restrict(
            is_vertical,
            blocks_in_band[peer_idx[1]],
            &block_masks_in_band[peer_idx[1]],
        )?;
        self.block_restrict(
            is_vertical,
            blocks_in_band[peer_idx[2]],
            &block_masks_in_band[peer_idx[2]],
        )?;
        Ok(())
    }

    fn block_restrict(
        &mut self,
        is_vertical: bool,
        block_idx: usize,
        mask: &Block,
    ) -> Result<(), ()> {
        let ref mut block = self.blocks[block_idx];
        if block.is_subset_of(&mask) {
            return Ok(());
        }

        let block_r = block_idx / 3;
        let block_c = block_idx % 3;
        let mut elimination = BlockEliminations(block.0 & !mask.0);

        let mut first = true;
        while block.eliminate(&elimination) || first {
            first = false;
            // println!(
            //     "block_restrict   {} {}  : {}",
            //     is_vertical as usize,
            //     block_idx,
            //     block
            //         .0
            //         .as_array()
            //         .map(|x| format!("{:>3}", format!("{:o}", x)))
            //         .join(" ")
            // );

            let counts = block.simd_count_ones();
            if counts.simd_lt(MINIMUM_COUNT_OF_CANDIDATES_IN_BLOCK).any() {
                return Err(());
            }

            let mut asserted = Self::naked_single(block, &counts);
            asserted |= &Self::hidden_single(block);

            // Eliminates all the other occurrences of the asserted candidates in the block
            const BLOCK_CELLS_MASK: u16x16 = u16x16::from_array([
                0xffff, 0xffff, 0xffff, 0, 0xffff, 0xffff, 0xffff, 0, 0xffff, 0xffff, 0xffff, 0, 0,
                0, 0, 0,
            ]);
            let asserted_cells = asserted.0 & BLOCK_CELLS_MASK;
            let mut reduce_or_in_row_and_col = asserted_cells;
            // +-------------+
            // | .  .  .  c2 |
            // | .  .  .  c5 |
            // | .  .  .  c8 |
            // | c6 c7 c8  . |
            // +-------------+
            reduce_or_in_row_and_col |= simd_swizzle!(
                asserted.0,
                [15, 15, 15, 2, 15, 15, 15, 6, 15, 15, 15, 10, 8, 9, 10, 15]
            );
            // +-------------+
            // | .  .  .  c1 |
            // | .  .  .  c4 |
            // | .  .  .  c7 |
            // | c3 c4 c5  . |
            // +-------------+
            reduce_or_in_row_and_col |= simd_swizzle!(
                asserted.0,
                [15, 15, 15, 1, 15, 15, 15, 5, 15, 15, 15, 9, 4, 5, 6, 15]
            );
            // +-------------+
            // | .  .  .  c0 |
            // | .  .  .  c3 |
            // | .  .  .  c6 |
            // | c0 c1 c2  . |
            // +-------------+
            reduce_or_in_row_and_col |= simd_swizzle!(
                asserted.0,
                [15, 15, 15, 0, 15, 15, 15, 4, 15, 15, 15, 8, 0, 1, 2, 15]
            );
            // Row elimination
            // row_i is the `recuded or` of the asserted cells in the i-th row
            // +---------------------+
            // | row0 row0 row0 row0 |
            // | row1 row1 row1 row1 |
            // | row2 row2 row2 row2 |
            // | .    .    .    .    |
            // +---------------------+
            elimination.0 = simd_swizzle!(
                reduce_or_in_row_and_col,
                [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15]
            );
            // Column elimination
            // col_i is the `recuded or` of the asserted cells in the i-th column
            // +------------------+
            // | col0 col1 col2 . |
            // | col0 col1 col2 . |
            // | col0 col1 col2 . |
            // | col0 col1 col2 . |
            // +------------------+
            elimination.0 |= simd_swizzle!(
                reduce_or_in_row_and_col,
                [12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15]
            );
            // Block elimination
            // +------------------+
            // | col1 col2 col0 . |
            // | col1 col2 col0 . |
            // | col1 col2 col0 . |
            // | .    .    .    . |
            // +------------------+
            elimination.0 |= simd_swizzle!(
                reduce_or_in_row_and_col,
                [13, 14, 12, 15, 13, 14, 12, 15, 13, 14, 12, 15, 15, 15, 15, 15]
            );
            // +------------------+
            // | col2 col0 col1 . |
            // | col2 col0 col1 . |
            // | col2 col0 col1 . |
            // | .    .    .    . |
            // +------------------+
            elimination.0 |= simd_swizzle!(
                reduce_or_in_row_and_col,
                [14, 12, 13, 15, 14, 12, 13, 15, 14, 12, 13, 15, 15, 15, 15, 15]
            );

            // Eliminate all the other candidates of the asserted_cells
            elimination.0 |= asserted_cells.simd_ne(u16x16::splat(0)).to_int().cast();
            elimination.0 ^= asserted_cells;

            // Asserting the negative triads in the block eliminates the configurations that contain the triads
            let mut eliminating_configurations =
                BandConfigurationEliminations::from_asserted_negative_triad(block_idx, &asserted);
            // Eliminating the negative triads in the block is asserting the positive triads, and eliminates the configurations that do not contain the triads
            eliminating_configurations |=
                BandConfigurationEliminations::from_eliminated_negative_triad(
                    block_idx,
                    &elimination,
                );

            let horizontal_band_elimination = BandConfigurationEliminations(u16x8::from_slice(
                &eliminating_configurations.as_array()[0..8],
            ));
            let vertical_band_elimination = BandConfigurationEliminations(u16x8::from_slice(
                &eliminating_configurations.as_array()[8..16],
            ));
            self.bands[0][block_r].eliminations |= &horizontal_band_elimination;
            self.bands[1][block_c].eliminations |= &vertical_band_elimination;
        }

        if is_vertical {
            self.band_elimination(false, block_r, block_c)?;
            self.band_elimination(true, block_c, block_r)?;
        } else {
            self.band_elimination(true, block_c, block_r)?;
            self.band_elimination(false, block_r, block_c)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn naked_single(block: &Block, counts: &u16x16) -> Block {
        // For each cell in the block, if there only remains one candidate, we can assert this candidate is in the cell.
        // This is also known as the "naked single" solving technique.
        let asserting_cells_mask: u16x16 = counts
            .simd_eq(MINIMUM_COUNT_OF_CANDIDATES_IN_BLOCK)
            .to_int()
            .cast();
        let asserting_cells = block.0 & asserting_cells_mask;
        return Block(asserting_cells);
    }

    #[inline(always)]
    fn hidden_single(block: &Block) -> Block {
        // For each row/column (including the negative triads) in the block, if there is a candidate that only appears in one place (a cell or a triad), we can assert the candidate is in the place.
        // This is also known as the "hidden single" solving technique.
        // The 3, 7, 11, 12, 13, 15-th element holds the candidates that appear in the row/column once or more.
        // +-------------+
        // | c0 c1 c2 H0 |
        // | c3 c4 c5 H1 |
        // | c6 c7 c8 H2 |
        // | V0 V1 V2  . |
        // +-------------+
        let mut one_or_more = block.0;
        // +-------------+
        // | .  .  .  c2 |
        // | .  .  .  c5 |
        // | .  .  .  c8 |
        // | c6 c7 c8  . |
        // +-------------+
        let mut rotate = simd_swizzle!(
            block.0,
            [15, 15, 15, 2, 15, 15, 15, 6, 15, 15, 15, 10, 8, 9, 10, 15]
        );
        // two_or_more_r = r3 & r2
        let mut two_or_more = one_or_more & rotate;
        // one_or_more_r = r3 | r2
        one_or_more |= rotate;
        // +-------------+
        // | .  .  .  c1 |
        // | .  .  .  c4 |
        // | .  .  .  c7 |
        // | c3 c4 c5  . |
        // +-------------+
        rotate = simd_swizzle!(
            block.0,
            [15, 15, 15, 1, 15, 15, 15, 5, 15, 15, 15, 9, 4, 5, 6, 15]
        );
        // two_or_more_r = (r3 & r2) | ((r3 | r2) & r1)
        two_or_more |= one_or_more & rotate;
        // one_or_more_r = r3 | r2 | r1
        one_or_more |= rotate;
        // +-------------+
        // | .  .  .  c0 |
        // | .  .  .  c3 |
        // | .  .  .  c6 |
        // | c0 c1 c2  . |
        // +-------------+
        rotate = simd_swizzle!(
            block.0,
            [15, 15, 15, 0, 15, 15, 15, 4, 15, 15, 15, 8, 0, 1, 2, 15]
        );
        // two_or_more_r = (r3 & r2) | ((r3 | r2) & r1) | ((r3 | r2 | r1) & r0)
        two_or_more |= one_or_more & rotate;
        // one_or_more_r = r3 | r2 | r1 | r0
        one_or_more |= rotate;
        let only_one = one_or_more ^ two_or_more;
        // +---------------------+
        // | row0 row0 row0 row0 |
        // | row1 row1 row1 row1 |
        // | row2 row2 row2 row2 |
        // | .    .    .    .    |
        // +---------------------+
        let horizontal_only_one_mask = simd_swizzle!(
            only_one,
            [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15]
        );
        // +------------------+
        // | col0 col1 col2 . |
        // | col0 col1 col2 . |
        // | col0 col1 col2 . |
        // | col0 col1 col2 . |
        // +------------------+
        let vertical_only_one_mask = simd_swizzle!(
            only_one,
            [12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15]
        );
        let asserting_cells = (horizontal_only_one_mask | vertical_only_one_mask) & block.0;
        return Block(asserting_cells);
    }

    pub fn solve(&mut self) -> Result<(), ()> {
        if let Some((is_vertical, band_idx, configuration_value_mask)) = self.choose_branch_point()
        {
            return self.branch(is_vertical, band_idx, configuration_value_mask);
        }
        Ok(())
    }

    fn choose_branch_point(&self) -> Option<(bool, usize, u16)> {
        fn count_ones(v: u16x8) -> u16 {
            unsafe { std::intrinsics::simd::simd_ctpop(v).reduce_sum() }
        }
        // Choose the unsolved band with the least number of configurations.
        // A band is already solved if there is only nine bits set in its configurations.
        let configuration_possibilities = [
            count_ones(self.bands[0][0].configurations.0).wrapping_sub(10),
            count_ones(self.bands[0][1].configurations.0).wrapping_sub(10),
            count_ones(self.bands[0][2].configurations.0).wrapping_sub(10),
            count_ones(self.bands[1][0].configurations.0).wrapping_sub(10),
            count_ones(self.bands[1][1].configurations.0).wrapping_sub(10),
            count_ones(self.bands[1][2].configurations.0).wrapping_sub(10),
        ];
        if let Some((index, _)) = configuration_possibilities
            .iter()
            .enumerate()
            .filter(|&(_, &v)| v < 256)
            .min_by_key(|(_, &v)| v)
        {
            let is_vertical = index >= 3;
            let band_idx = index % 3;
            let ref configuration = self.bands[is_vertical as usize][band_idx].configurations;
            // Choose one undetermined digit with the least number of possibilities.
            // 0
            let mut rotated = configuration.0.rotate_elements_left::<1>();
            let mut one_or_more = configuration.0;
            // 0 & 1
            let mut two_or_more = one_or_more & rotated;
            // 0 | 1
            one_or_more |= rotated;
            rotated = configuration.0.rotate_elements_left::<2>();
            // 0 & 1 & 2
            let mut three_or_more = two_or_more & rotated;
            // (0 & 1) | ((0 | 1) & 2)
            two_or_more |= one_or_more & rotated;
            // 0 | 1 | 2
            one_or_more |= rotated;
            rotated = configuration.0.rotate_elements_left::<3>();
            let mut four_or_more = three_or_more & rotated;
            // (0 & 1 & 2) | ((0 & 1) | ((0 | 1) & 2)) & 3
            three_or_more |= two_or_more & rotated;
            // (0 & 1) | ((0 | 1) & 2) | ((0 | 1 | 2) & 3)
            two_or_more |= one_or_more & rotated;
            // 0 | 1 | 2 | 3
            one_or_more |= rotated;
            rotated = configuration.0.rotate_elements_left::<4>();
            four_or_more |= three_or_more & rotated;
            three_or_more |= two_or_more & rotated;
            two_or_more |= one_or_more & rotated;
            one_or_more |= rotated;
            rotated = configuration.0.rotate_elements_left::<5>();
            four_or_more |= three_or_more & rotated;
            two_or_more |= one_or_more & rotated;
            one_or_more |= rotated;

            let only_two = two_or_more.as_array()[0] ^ three_or_more.as_array()[0];
            let only_three = three_or_more.as_array()[0] ^ four_or_more.as_array()[0];
            if only_two != 0 {
                let lowest_bit = only_two & (!only_two + 1);
                return Some((is_vertical, band_idx, lowest_bit));
            } else if only_three != 0 {
                let lowest_bit = only_three & (!only_three + 1);
                return Some((is_vertical, band_idx, lowest_bit));
            } else {
                let four_or_more = four_or_more.as_array()[0];
                let lowest_bit = four_or_more & (!four_or_more + 1);
                return Some((is_vertical, band_idx, lowest_bit));
            }
        }

        None
    }

    fn branch(
        &mut self,
        is_vertical: bool,
        band_idx: usize,
        configuration_value_mask: u16,
    ) -> Result<(), ()> {
        let candidates = self.bands[is_vertical as usize][band_idx].configurations.0
            & u16x8::splat(configuration_value_mask);

        // Try to eliminate one of the configurations and see if the board is still solvable.
        let mut state_copy = self.clone();
        let has_values = candidates.simd_ne(u16x8::splat(0)).to_array();
        let mut configurations = None;
        for i in 0..8 {
            if has_values[i] {
                configurations = Some(u16x8::from_array(array::from_fn(|j| {
                    if i == j {
                        0
                    } else {
                        candidates.as_array()[j]
                    }
                })));
                break;
            }
        }
        let configurations = configurations.unwrap();
        state_copy.bands[is_vertical as usize][band_idx]
            .eliminations
            .0 |= configurations;
        if state_copy
            .band_elimination(is_vertical, band_idx, 0)
            .is_ok()
        {
            if state_copy.solve().is_ok() {
                *self = state_copy;
                return Ok(());
            }
        }

        // Try to assert the configuration and see if the board is still solvable.
        self.bands[is_vertical as usize][band_idx].eliminations.0 |= candidates ^ configurations;
        if self.band_elimination(is_vertical, band_idx, 0).is_ok() {
            return self.solve();
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_candidates(state: &State) {
        for i in 0..81 {
            let block_index = BlockIndex::from_cell(i as u8);
            let bits = state.blocks[block_index.block_idx as usize].0.as_array()
                [block_index.element_idx as usize];
            print!(
                "{:9} ",
                (0..9)
                    .filter(|&j| (1 << j) & bits != 0)
                    .map(|j| j + 1)
                    .join("")
            );
            if i % 9 == 8 {
                println!("");
            }
        }
    }

    fn print_values(state: &State) -> String {
        let mut result = String::new();
        for i in 0..81 {
            let block_index = BlockIndex::from_cell(i as u8);
            result.push_str(&format!(
                "{}",
                state.blocks[block_index.block_idx as usize]
                    .0
                    .trailing_zeros()
                    .as_array()[block_index.element_idx as usize]
                    + 1
            ));
        }
        result
    }

    #[test]
    fn test_state_from_values() {
        let mut state = State::from_values(
            "6.....3...5..9..8...2..6..98.....7...7..5..4......1..51..3..5...4..2..6...8..7..2",
        );
        println!("");
        state.solve();
        debug_assert_eq!(
            print_values(&state),
            "689514327457293681312876459835942716971658243264731895126389574743125968598467132"
        );
        println!("");
    }
}

use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::{CellIndex, CellValue};

pub fn solve_guess(sudoku: &SudokuSolver, recorder: &mut SolutionRecorder) {
    let mut state = State::from_values(&sudoku.sudoku().to_value_string());
    state.solve();
    for i in 0..81 {
        let block_index = BlockIndex::from_cell(i as u8);
        if sudoku.sudoku().get_cell_value(i as CellIndex).is_some() {
            continue;
        }
        let bits = state.blocks[block_index.block_idx as usize].0.as_array()
            [block_index.element_idx as usize];
        if bits.count_ones() == 1 {
            let value = bits.trailing_zeros() + 1;
            recorder.add_value_set(
                Technique::Guess,
                "".to_string(),
                i as CellIndex,
                value as CellValue,
            );
            if recorder.should_return() {
                return;
            }
        }
    }
}
