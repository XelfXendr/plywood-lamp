use core::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct RangesError;

pub struct OverlapRanges<T: Eq + Ord, const N: usize> {
    ranges: [T; N],
}

impl<T: Eq + Ord, const N: usize> Index<usize> for OverlapRanges<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.ranges[index % N]
    }
}

impl<T: Eq + Ord, const N: usize> IndexMut<usize> for OverlapRanges<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.ranges[index % N]
    }
}

impl<T: Eq + Ord, const N: usize> OverlapRanges<T, N> {
    pub fn new(ranges: [T; N]) -> Result<Self, RangesError> {
        // ensure values are well ordered
        for i in 0..N - 1 {
            if ranges[i] >= ranges[i + 1] {
                Err(RangesError)?
            }
        }

        Ok(Self { ranges })
    }


    pub fn which(&self, value: T) -> usize {
        
        if value < self.ranges[0] || value >= self.ranges[N-1] {
            return 0
        }


        for i in 0..N - 1 {
            if self.ranges[i] <= value && value < self.ranges[i+1] {
                return i+1
            }
        }

        unreachable!("because the array is well ordered")
    }
}
