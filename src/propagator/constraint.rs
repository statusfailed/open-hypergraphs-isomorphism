use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, PartialEq, Debug)]
pub enum Constraint<T: Hash + Eq> {
    Any,
    Set(HashSet<T>),
}

impl<T: Hash + Eq> Constraint<T> {
    pub fn single(x: T) -> Constraint<T> {
        Constraint::Set(HashSet::from([x]))
    }
}

impl<T: Clone + Hash + Eq> Constraint<T> {
    pub fn intersection(&mut self, s: HashSet<T>) {
        *self = match self {
            Self::Any => Self::Set(s),
            Self::Set(t) => Self::Set(s.intersection(&t).cloned().collect()),
        };
    }

    pub fn intersect_one(&mut self, x: T) {
        self.intersection(HashSet::from([x]));
    }

    /// NOTE: we assume the represented set is nonempty here!
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Any => false,
            Self::Set(s) => s.is_empty(),
        }
    }
}
