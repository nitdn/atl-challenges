use std::{
    cmp::{Ordering, PartialOrd},
    iter::{self, Sum},
    marker::Copy,
};
#[derive(Debug)]
pub struct Heavy<'a, T>
where
    T: Sum + PartialOrd + Copy + Into<u64>,
{
    weights: &'a [T],
    countdown: u64,
}

impl<'a, T> Heavy<'a, T>
where
    T: Sum + PartialOrd + Copy + Into<u64>,
{
    pub fn new(weights: &'a [T], countdown: u64) -> Self {
        Self { weights, countdown }
    }
    pub fn weigh<I>(self, lhs: I, rhs: I) -> (Option<Ordering>, Self)
    where
        I: iter::Iterator<Item = usize>,
    {
        if self.countdown == 0 {
            (None, self)
        } else {
            let lhs_sum: u64 = lhs.map(|index| self.weights[index].into()).sum();
            let rhs_sum: u64 = rhs.map(|index| self.weights[index].into()).sum();
            let self_new = Self {
                countdown: self.countdown - 1,
                ..self
            };

            (lhs_sum.partial_cmp(&rhs_sum), self_new)
        }
    }
}

pub fn find_largest<T>(heavy: Heavy<T>, start: usize, len: usize) -> Option<usize>
where
    T: Sum + PartialOrd + Copy + Into<u64>,
{
    if len <= 1 {
        Some(start)
    } else {
        let chunk_size = len.div_ceil(3);
        let left_end = start + chunk_size;
        let left = start..left_end;
        let mid_end = left_end + chunk_size;
        let mid = left_end..mid_end;
        let right_end = mid_end + chunk_size;
        dbg!(&left, &mid, mid_end..right_end);
        let (Some(cmp), heavy) = heavy.weigh(left, mid) else {
            return None;
        };
        match cmp {
            Ordering::Less => find_largest(heavy, left_end, chunk_size),
            Ordering::Equal => find_largest(heavy, mid_end, chunk_size),
            Ordering::Greater => find_largest(heavy, start, chunk_size),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_breakage() {
        let weights = vec![1u8, 1, 1, 1, 1, 1, 1, 2];
        let countdown = 4;
        let heavy = Heavy::new(&weights, countdown);

        let new_heavy = heavy.weigh([0].into_iter(), [0].into_iter());
        assert_eq!(new_heavy.0, Some(Ordering::Equal));
        let new_heavy = new_heavy
            .1
            .weigh([0].into_iter(), [0].into_iter())
            .1
            .weigh(0..1, 0..1)
            .1
            .weigh([0].iter().copied(), [0].iter().copied())
            .1
            .weigh(0..0, 0..0);
        assert_eq!(new_heavy.0, None);
    }

    #[test]
    fn actually_find_the_biggest() {
        let weights = vec![1u8, 3, 1, 1, 1, 1, 1, 1];
        let countdown = 4;
        let heavy = Heavy::new(&weights, countdown);
        assert_eq!(find_largest(heavy, 0, 8), Some(1));
        let countdown = 3;
        let heavy = Heavy::new(&weights, countdown);
        assert_eq!(find_largest(heavy, 0, 8), Some(1));
        let countdown = 2;
        let heavy = Heavy::new(&weights, countdown);
        assert_eq!(find_largest(heavy, 0, 8), Some(1));
    }

    #[test]
    fn find_biggest_for_arbitary_n() {
        const LENGTH: usize = 3491900090;
        // const LENGTH: usize = 3491900090;
        let mut weights = vec![2u8; LENGTH];
        let big_index = 41;
        weights[big_index] = 23;
        let countdown = LENGTH as u64 / 2;
        let heavy = Heavy::new(&weights, countdown);
        assert_eq!(find_largest(heavy, 0, LENGTH), Some(big_index));
        let countdown = (LENGTH as f64).powf(1.0 / 3.0).ceil() as u64;
        let heavy = Heavy::new(&weights, countdown);
        assert_eq!(find_largest(heavy, 0, LENGTH), Some(big_index));
    }
}
