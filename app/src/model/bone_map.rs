use crate::model::BoneKind;

use core::ops::{Index, IndexMut};
use stackvec::{error::IncompleteArrayError, TryCollect, TryFromIterator};
use std::collections::HashMap;
use std::iter::{Enumerate, Map};

/// Provides a map of `BoneKind` -> `T`.
#[derive(Debug, Default)]
pub struct BoneMap<T>([T; BoneKind::num_types()]);
impl<T> BoneMap<T> {
    pub fn new(map: [T; BoneKind::num_types()]) -> Self {
        Self(map)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }
}

// ---- Type conversion stuff ----
impl<T> TryFrom<HashMap<BoneKind, T>> for BoneMap<T> {
    type Error = IncompleteArrayError;

    fn try_from(other: HashMap<BoneKind, T>) -> Result<Self, Self::Error> {
        other.into_iter().try_collect()
    }
}

impl<T> TryFromIterator<(BoneKind, T)> for BoneMap<T> {
    type Error = IncompleteArrayError;

    fn try_from_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = (BoneKind, T)>,
    {
        let mut bmap: BoneMap<Option<T>> = BoneMap::default();
        for (kind, item) in iter.into_iter().take(BoneKind::NUM_TYPES) {
            bmap[kind] = Some(item);
        }
        if bmap.iter().any(|(_kind, item)| item.is_none()) {
            return Err(IncompleteArrayError);
        }
        Ok(BoneMap::new(bmap.0.map(|item| item.unwrap())))
    }
}

// ---- Index stuff ----
impl<T> Index<BoneKind> for BoneMap<T> {
    type Output = T;

    fn index(&self, index: BoneKind) -> &Self::Output {
        // I *could* do get_unchecked, but meh, why introduce more unsafe. Maybe the
        // compiler will optimize it.
        &self.0[usize::from(index)]
    }
}
impl<T> IndexMut<BoneKind> for BoneMap<T> {
    fn index_mut(&mut self, index: BoneKind) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}

// ---- Iterator stuff ----

type MapIdxFnType<T> = fn((usize, T)) -> (BoneKind, T);

pub type Iter<'a, T> = Map<Enumerate<std::slice::Iter<'a, T>>, MapIdxFnType<&'a T>>;

pub type IterMut<'a, T> = Map<Enumerate<std::slice::IterMut<'a, T>>, MapIdxFnType<&'a mut T>>;

pub type IntoIter<T> =
    Map<Enumerate<std::array::IntoIter<T, { BoneKind::NUM_TYPES }>>, MapIdxFnType<T>>;

impl<T> IntoIterator for BoneMap<T> {
    type Item = (BoneKind, T);

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().enumerate().map(map_idx)
    }
}

impl<'a, T> IntoIterator for &'a BoneMap<T> {
    type Item = (BoneKind, &'a T);

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().enumerate().map(map_idx)
    }
}

impl<'a, T> IntoIterator for &'a mut BoneMap<T> {
    type Item = (BoneKind, &'a mut T);

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut().enumerate().map(map_idx)
    }
}

fn map_idx<T>(item: (usize, T)) -> (BoneKind, T) {
    (BoneKind::try_from(item.0).unwrap(), item.1)
}
