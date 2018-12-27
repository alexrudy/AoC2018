use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub(crate) struct RepeatedElementResult<T> {
    first: T,
    last: T,
    length: usize,
    start: usize,
}

impl<T> RepeatedElementResult<T> {
    pub(crate) fn first(&self) -> &T {
        &self.first
    }

    pub(crate) fn last(&self) -> &T {
        &self.last
    }

    pub(crate) fn length(&self) -> usize {
        self.length
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }
}

pub(crate) fn repeated_element<I, T>(iter: I) -> Option<RepeatedElementResult<T>>
where
    I: Iterator<Item = T>,
    T: Hash + Eq + Ord + Clone,
{
    let mut seen = HashMap::new();
    let mut last = None;

    for (i, item) in iter.enumerate() {
        if let Some(s) = seen.insert(item.clone(), i) {
            let last = last.unwrap_or_else(|| item.clone());
            return Some(RepeatedElementResult {
                first: item,
                last: last,
                length: i - s,
                start: s,
            });
        } else {
            last = Some(item);
        }
    }
    None
}
