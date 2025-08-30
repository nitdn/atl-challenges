use std::cmp::Ordering;
#[derive(Debug)]
pub struct Heavy {
    weights: Vec<i64>,
    countdown: u64,
}

impl Heavy {
    pub fn new(weights: Vec<i64>, countdown: u64) -> Self {
        Self { weights, countdown }
    }
    pub fn weigh(self, lhs: &[usize], rhs: &[usize]) -> (Option<Ordering>, Self) {
        if self.countdown == 0 {
            (None, self)
        } else {
            let lhs_sum: i64 = lhs.iter().map(|index| self.weights[*index]).sum();
            let rhs_sum: i64 = rhs.iter().map(|index| self.weights[*index]).sum();
            let self_new = Self {
                countdown: self.countdown - 1,
                ..self
            };

            (lhs_sum.partial_cmp(&rhs_sum), self_new)
        }
    }
}

pub fn generate_index_list(slice: &[i64]) -> Vec<usize> {
    let mut index_list = Vec::new();
    for (index, _) in slice.iter().enumerate() {
        index_list.push(index);
    }
    index_list
}

pub fn find_largest(heavy: Heavy, index_list: &[usize]) -> Option<usize> {
    if index_list.len() <= 1 {
        index_list.first().copied()
    } else {
        let chunk_size = 1.max(index_list.len().div_ceil(3));
        let chunks: Vec<_> = index_list.chunks(chunk_size).collect();
        dbg!(&chunks);
        let (Some(cmp), heavy) = heavy.weigh(chunks[0], chunks[1]) else {
            return None;
        };
        match cmp {
            Ordering::Less => find_largest(heavy, chunks[1]),
            Ordering::Equal => find_largest(heavy, chunks[2]),
            Ordering::Greater => find_largest(heavy, chunks[0]),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_breakage() {
        let weights = vec![1, 1, 1, 1, 1, 1, 1, 2];
        let countdown = 4;
        let heavy = Heavy::new(weights, countdown);

        let new_heavy = heavy.weigh(&[0], &[0]);
        assert_eq!(new_heavy.0, Some(Ordering::Equal));
        let new_heavy = new_heavy
            .1
            .weigh(&[0], &[0])
            .1
            .weigh(&[0], &[0])
            .1
            .weigh(&[0], &[0])
            .1
            .weigh(&[0], &[0]); // the 5th one should fail
        assert_eq!(new_heavy.0, None);
    }

    #[test]
    fn check_index_list() {
        let weights = vec![1, 3, 1, 1, 1, 1, 1, 1];
        let index_list = vec![0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(index_list, generate_index_list(&weights));
    }

    #[test]
    fn actually_find_the_biggest() {
        let weights = vec![1, 3, 1, 1, 1, 1, 1, 1];
        let index_list = [0, 1, 2, 3, 4, 5, 6, 7];
        let countdown = 4;
        let heavy = Heavy::new(weights.clone(), countdown);
        assert_eq!(find_largest(heavy, &index_list), Some(1));
        let countdown = 3;
        let heavy = Heavy::new(weights.clone(), countdown);
        assert_eq!(find_largest(heavy, &index_list), Some(1));
        let countdown = 2;
        let heavy = Heavy::new(weights.clone(), countdown);
        assert_eq!(find_largest(heavy, &index_list), Some(1));
    }

    #[test]
    fn find_biggest_for_arbitary_n() {
        let length = 88;
        let mut weights = vec![2; length];
        let big_index = 41;
        weights[big_index] = 23;
        let index_list = generate_index_list(&weights);
        let countdown = length as u64 / 2;
        let heavy = Heavy::new(weights.clone(), countdown);
        assert_eq!(find_largest(heavy, &index_list), Some(big_index));
        let countdown = (length as f64).powf(1.0 / 3.0).ceil() as u64;
        let heavy = Heavy::new(weights, countdown);
        assert_eq!(find_largest(heavy, &index_list), Some(big_index));
    }
}
