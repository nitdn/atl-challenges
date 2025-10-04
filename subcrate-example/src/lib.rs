pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn test_random_spaces() -> u64 {
    // test for bad spaces, will not compile
    // if true {
    //     unimplemented!()
    // }
    unimplemented!()
}

pub fn test_pattern_matching((1 | 2 | 3 | _): u64) {
    unimplemented!("why did i do this")
}

pub fn merge_sort<Orderable: Ord + Copy + std::fmt::Debug>(unmerged: &mut [Orderable]) {
    match unmerged.len() {
        2 if unmerged[0] > unmerged[1] => {
            unmerged.swap(0, 1);
        }
        0..=2 => {}
        len @ 3.. => {
            let (lo, hi) = unmerged.split_at_mut(len >> 1);
            merge_sort(lo);
            merge_sort(hi);
            swap_elements(lo, hi)
        }
    }
}

fn swap_elements<Orderable: Ord + Copy + std::fmt::Debug>(
    lo: &mut [Orderable],
    hi: &mut [Orderable],
) {
    // if lo.is_empty() || hi.is_empty() || lo.last().unwrap() <= hi.first().unwrap() {
    //     return [lo, hi].concat();
    // }
    let mut merged = Vec::new();
    let mut lo_iter = lo.iter().peekable();
    let mut hi_iter = hi.iter().peekable();

    while let (Some(&&lo_val), Some(&&hi_val)) = (lo_iter.peek(), hi_iter.peek()) {
        if lo_val >= hi_val {
            merged.push(hi_val);
            hi_iter.next();
        } else {
            merged.push(lo_val);
            lo_iter.next();
        }
    }
    for i in lo_iter {
        merged.push(*i);
    }
    for i in hi_iter {
        merged.push(*i);
    }

    for item in lo {
        *item = merged.remove(0);
    }
    for item in hi {
        *item = merged.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn merger_sort() {
        let mut unmerged: Vec<_> = (0..20).rev().collect();
        merge_sort(&mut unmerged);
        dbg!(&unmerged);
        assert!(unmerged.is_sorted())
    }
}
