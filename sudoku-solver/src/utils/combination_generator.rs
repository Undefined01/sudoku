use crate::solver::return_if_some;
use crate::sudoku::{CellValue, Step, StepRule};
use crate::utils::{CellSet, NamedCellSet};
use crate::SudokuSolver;

use std::iter::FromIterator;

use arrayvec::ArrayVec;

const MAX_SIZE: usize = 4;

pub struct CombinationOptions<'a> {
    pub on_element_selected: Option<&'a mut dyn FnMut(usize, usize) -> bool>,
    pub on_element_unselected: Option<&'a mut dyn FnMut(usize, usize)>,
}

impl<'a> Default for CombinationOptions<'a> {
    fn default() -> Self {
        Self {
            on_element_selected: None,
            on_element_unselected: None,
        }
    }
}

pub struct CombinationIterator<'a, T: Copy> {
    arr: &'a [T],
    n: usize,
    k: usize,
    options: CombinationOptions<'a>,
    stack: ArrayVec<usize, MAX_SIZE>,
    result: ArrayVec<T, MAX_SIZE>,
}

impl<'a, T: Copy> CombinationIterator<'a, T> {
    #[inline(always)]
    pub fn new(arr: &'a [T], k: usize, options: CombinationOptions<'a>) -> Self {
        debug_assert!(k <= MAX_SIZE);

        let stack = ArrayVec::<usize, MAX_SIZE>::new();
        let result = ArrayVec::<T, MAX_SIZE>::new();
        let n = arr.len();
        Self {
            arr,
            n,
            k,
            options,
            stack,
            result,
        }
    }

    #[inline(always)]
    fn try_update(&mut self, current: usize) -> Option<&'a [T]> {
        if let Some(ref mut on_element_selected) = self.options.on_element_selected {
            if !on_element_selected(current, self.stack[current]) {
                return None;
            }
        }
        self.result.push(self.arr[self.stack[current]]);
        for i in current + 1..self.k {
            self.stack.push(self.stack[i - 1] + 1);
            if let Some(ref mut on_element_selected) = self.options.on_element_selected {
                if !on_element_selected(i, self.stack[i]) {
                    return None;
                }
            }
            self.result.push(self.arr[self.stack[i]]);
        }

        // Some(self.result.as_slice())
        unsafe { Some(&*(self.result.as_slice() as *const [T])) }
    }
}

impl<'a, T: Copy> Iterator for CombinationIterator<'a, T> {
    type Item = &'a [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let mut skip_unselected = false;
        if self.stack.len() == 0 {
            self.stack.push(0);
            return_if_some!(self.try_update(0));
            skip_unselected = true;
        }

        while let Some(&current_element) = self.stack.last() {
            let stack_index = self.stack.len() - 1;
            if current_element + (self.k - stack_index) >= self.n {
                self.stack.pop().unwrap();
                if skip_unselected {
                    skip_unselected = false;
                } else {
                    self.result.pop().unwrap();
                    if let Some(on_element_unselected) = self.options.on_element_unselected.as_mut()
                    {
                        on_element_unselected(stack_index, current_element);
                    }
                }
                continue;
            }

            if skip_unselected {
                skip_unselected = false;
            } else {
                self.result.pop().unwrap();
                if let Some(on_element_unselected) = self.options.on_element_unselected.as_mut() {
                    on_element_unselected(stack_index, current_element);
                }
            }

            *self.stack.last_mut().unwrap() += 1;
            return_if_some!(self.try_update(stack_index));
            skip_unselected = true;
        }

        None
    }
}

pub fn combinations<'a, T: Copy>(
    arr: &'a [T],
    k: usize,
    options: CombinationOptions<'a>,
) -> CombinationIterator<'a, T> {
    CombinationIterator::new(arr, k, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_combination_iterator() {
        let arr = [1, 2, 3, 4, 5];
        let options = CombinationOptions {
            on_element_selected: None,
            on_element_unselected: None,
        };
        let iter = CombinationIterator::new(&arr, 2, options);
        let result = iter
            .map(|s| s.iter().cloned().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let expected = arr.iter().copied().combinations(2).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_combination_iterator_options() {
        let arr = [1, 2, 3, 4, 5];
        let mut selected_order = vec![];
        let mut unselected_order = vec![];
        let ref mut on_element_selected = |pos, element| {
            if element != 3 {
                selected_order.push(element);
                return true;
            }
            false
        };
        let ref mut on_element_unselected = |pos, element| {
            unselected_order.push(element);
        };
        let options = CombinationOptions {
            on_element_selected: Some(on_element_selected),
            on_element_unselected: Some(on_element_unselected),
        };
        let iter = CombinationIterator::new(&arr, 2, options);
        let result = iter
            .map(|s| s.iter().cloned().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let expected = arr
            .iter()
            .copied()
            .filter(|&x| x != 4)
            .combinations(2)
            .collect::<Vec<_>>();
        assert_eq!(result, expected);
        assert_eq!(selected_order, [0, 1, 2, 4, 1, 2, 4, 2, 4]);
        assert_eq!(unselected_order, [1, 2, 4, 0, 2, 4, 1, 4, 2]);
    }
}
