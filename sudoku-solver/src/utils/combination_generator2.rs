use std::sync::{Arc, LazyLock};

use arrayvec::ArrayVec;
use itertools::Itertools;

const MAX_ARRAY_LEN: usize = 9;
const MAX_SIZE: usize = 4;

static CACHE: LazyLock<Vec<Vec<Arc<Vec<Vec<usize>>>>>> = LazyLock::new(|| {
    (0..=MAX_ARRAY_LEN)
        .map(|length| {
            (0..=length.min(MAX_SIZE))
                .map(|size| Arc::new((0..length).combinations(size).collect_vec()))
                .collect_vec()
        })
        .collect_vec()
});

pub fn combinations<'a, T: Copy>(arr: &'a [T], size: usize) -> CombinationIterator<'a, T> {
    debug_assert!(arr.len() <= MAX_ARRAY_LEN);
    debug_assert!(size <= MAX_SIZE);

    if arr.len() < size {
        return CombinationIterator {
            combination_cache: CACHE[0][0].clone(),
            arr,
            idx: usize::MAX,
        };
    }

    let combination_cache = CACHE[arr.len()][size].clone();
    CombinationIterator {
        combination_cache,
        arr,
        idx: 0,
    }
}

pub struct CombinationIterator<'a, T> {
    combination_cache: Arc<Vec<Vec<usize>>>,
    arr: &'a [T],
    idx: usize,
}

impl<'a, T: Copy> Iterator for CombinationIterator<'a, T> {
    type Item = ArrayVec<T, MAX_SIZE>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.combination_cache.len() - self.idx;
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.combination_cache.len() {
            return None;
        }
        let mut combination = ArrayVec::new();
        for &element in &self.combination_cache[self.idx] {
            combination.push(self.arr[element]);
        }
        self.idx += 1;
        Some(combination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combination_generator() {
        for len in 0..=MAX_ARRAY_LEN {
            for size in 0..=MAX_SIZE {
                let arr: Vec<u8> = (0..len as u8).collect();
                let combinations: Vec<ArrayVec<u8, MAX_SIZE>> = combinations(&arr, size).collect();
                let expected: Vec<Vec<u8>> = arr.iter().copied().combinations(size).collect();
                assert_eq!(combinations.len(), expected.len());
                for (combination, expected) in combinations.iter().zip(expected.iter()) {
                    assert_eq!(combination.as_slice(), expected.as_slice());
                }
            }
        }
    }
}
