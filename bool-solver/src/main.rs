use itertools::Itertools;

fn solve(inputs: &[&str], f: impl Fn(&[bool]) -> bool) {
    for input in inputs
        .iter()
        .map(|&var| [true, false].map(|boolean| (var, boolean)))
        .multi_cartesian_product()
        .filter(|arg| {
            let (_, fields): (Vec<_>, Vec<_>) = arg.iter().copied().unzip();
            f(&fields)
        })
    {
        println!("{input:?}",);
    }
}

fn main() {
    solve(
        &["a", "b", "c"],
        |input: &[bool]| matches!(input, &[a, b, c] if a ^ b ^ c),
    );
}
