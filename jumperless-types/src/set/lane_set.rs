use core::iter::{Enumerate, FilterMap};

use crate::Lane;

pub struct LaneSet<'a>(&'a [Lane], [u8; 16]);

impl<'a> LaneSet<'a> {
    /// Construct a new LaneSet holding the given lanes
    pub fn new(lanes: &'a [Lane]) -> Self {
        assert!(lanes.len() < 128);
        Self(lanes, [0xFF; 16])
    }

    /// Remove the first lane that matches given predicate and return it
    pub fn take<F: Fn(Lane) -> bool>(&mut self, predicate: F) -> Option<Lane> {
        for i in 0..self.0.len() {
            if self.has_index(i) {
                let lane = self.0[i];
                if predicate(lane) {
                    self.clear_index(i);
                    return Some(lane);
                }
            }
        }
        None
    }

    pub fn has_index(&self, index: usize) -> bool {
        let (i, j) = (index / 8, index % 8);
        (self.1[i] >> j) & 1 == 1
    }

    pub fn clear_index(&mut self, index: usize) {
        let (i, j) = (index / 8, index % 8);
        self.1[i] &= !(1 << j);
    }

    pub fn iter(&'a self) -> impl Iterator<Item = Lane> + 'a {
        self.0.iter().enumerate().filter_map(
            |(i, lane)| {
                if self.has_index(i) {
                    Some(*lane)
                } else {
                    None
                }
            },
        )
    }
}

// pub type ClosureType<'a> = impl FnMut((usize, &'a Lane)) -> Option<Lane>;

impl<'a> core::iter::IntoIterator for LaneSet<'a> {
    type Item = Lane;

    type IntoIter = FilterMap<
        Enumerate<core::slice::Iter<'a, Lane>>,
        impl FnMut((usize, &'a Lane)) -> Option<Lane>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().enumerate().filter_map(
            move |(i, lane)| {
                if self.has_index(i) {
                    Some(*lane)
                } else {
                    None
                }
            },
        )
    }
}
//     // fn into_iter(self) -> impl Iterator<Item = Lane> + 'a {
//     //     self.0.iter().enumerate().filter_map(
//     //         move |(i, lane)| {
//     //             if self.has_index(i) {
//     //                 Some(*lane)
//     //             } else {
//     //                 None
//     //             }
//     //         },
//     //     )
//     // }
// }
