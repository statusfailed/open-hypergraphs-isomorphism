use std::fmt::Debug;

#[derive(Clone, PartialEq, Debug)]
pub struct Permutation(Vec<usize>);

impl Permutation {
    pub fn new(values: impl IntoIterator<Item = usize>) -> Option<Self> {
        let vec: Vec<usize> = values.into_iter().collect();

        if vec.is_empty() {
            return Some(Self(vec));
        }

        let n = vec.len();
        let mut seen = vec![false; n];

        for &value in &vec {
            if value >= n || seen[value] {
                return None;
            }
            seen[value] = true;
        }

        if seen.iter().all(|&x| x) {
            Some(Self(vec))
        } else {
            None
        }
    }

    pub fn identity(size: usize) -> Self {
        Self((0..size).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_permutations() {
        assert!(Permutation::new([0, 1, 2]).is_some());
        assert!(Permutation::new([2, 0, 1]).is_some());
        assert!(Permutation::new([1, 0]).is_some());
        assert!(Permutation::new(vec![0]).is_some());
        assert!(Permutation::new(Vec::<usize>::new()).is_some());
    }

    #[test]
    fn test_invalid_permutations() {
        assert!(Permutation::new([0, 2]).is_none());
        assert!(Permutation::new([0, 1, 1]).is_none());
        assert!(Permutation::new([0, 1, 3]).is_none());
        assert!(Permutation::new([1, 2, 3]).is_none());
        assert!(Permutation::new([0, 0, 1]).is_none());
    }

    #[test]
    fn test_from_different_iterables() {
        assert!(Permutation::new(vec![0, 1, 2]).is_some());
        assert!(Permutation::new([0, 1, 2]).is_some());
        assert!(Permutation::new((0..3).collect::<Vec<_>>()).is_some());
        assert!(Permutation::new([2, 1, 0].iter().copied()).is_some());
    }
}
