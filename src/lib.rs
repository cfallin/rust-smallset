// smallset: a Rust crate for small unordered sets of elements, built on top of
// `smallvec`.
//
// Copyright (c) 2016 Chris Fallin <cfallin@c1f.net>. Released under the MIT license.
//

extern crate smallvec;

use std::fmt;
use std::iter::{FromIterator, IntoIterator};

use smallvec::{Array, SmallVec};
use std::collections::HashSet;
use std::hash::Hash;

/// A `SmallSet` is an unordered set of elements. It is designed to work best
/// for very small sets (no more than ten or so elements). In order to support
/// small sets very efficiently, it stores elements in a simple unordered array.
/// When the set is smaller than the size of the array `A`, all elements are
/// stored inline, without heap allocation. This is accomplished by using a
/// `smallvec::SmallVec`.
///
/// The insert, remove, and query methods on `SmallSet` have `O(n)` time
/// complexity in the current set size: they perform a linear scan to determine
/// if the element in question is present. This is inefficient for large sets,
/// but fast and cache-friendly for small sets.
///
/// Example usage:
///
/// ```
/// use smallset::SmallSet;
///
/// // `s` and its elements will be completely stack-allocated in this example.
/// let mut s: SmallSet<[u32; 4]> = SmallSet::new();
/// s.insert(1);
/// s.insert(2);
/// s.insert(3);
/// assert_eq!(s.len(), 3);
/// assert!(s.contains(&1));
/// ```

pub struct SmallSet<A: Array>
where
    A::Item: PartialEq + Eq,
{
    inner: InnerSmallVec<A>,
}

impl<A: Array> Default for SmallSet<A>
where
    A::Item: PartialEq + Eq + Hash,
{
    fn default() -> Self {
        SmallSet::new()
    }
}

pub enum InnerSmallVec<A: Array>
where
    A::Item: PartialEq + Eq,
{
    Stack(SmallVec<A>),
    Heap(std::collections::HashSet<A::Item>),
}

impl<A: Array> Default for InnerSmallVec<A>
where
    A::Item: PartialEq + Eq,
{
    fn default() -> Self {
        InnerSmallVec::Stack(SmallVec::new())
    }
}

impl<A: Array> Clone for InnerSmallVec<A>
where
    A::Item: PartialEq + Eq + Clone,
{
    fn clone(&self) -> Self {
        match &self {
            InnerSmallVec::Stack(elements) => InnerSmallVec::Stack(elements.clone()),
            InnerSmallVec::Heap(elements) => InnerSmallVec::Heap(elements.clone()),
        }
    }
}

impl<A: Array> PartialEq for SmallSet<A>
where
    A::Item: Eq + PartialEq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        fn set_same<A: Array>(stack: &SmallVec<A>, heap: &HashSet<A::Item>) -> bool
        where
            A::Item: Eq + PartialEq,
        {
            stack.len() == heap.len() && heap.iter().all(|x| stack.contains(x))
        }

        match (&self.inner, &other.inner) {
            (InnerSmallVec::Stack(lhs), InnerSmallVec::Stack(rhs)) => lhs.eq(rhs),
            (InnerSmallVec::Heap(lhs), InnerSmallVec::Heap(rhs)) => lhs.eq(rhs),
            (InnerSmallVec::Stack(stack), InnerSmallVec::Heap(heap)) => set_same(stack, heap),
            (InnerSmallVec::Heap(heap), InnerSmallVec::Stack(stack)) => set_same(stack, heap),
        }
    }
}

impl<A: Array> SmallSet<A>
where
    A::Item: PartialEq + Eq + Hash,
{
    /// Creates a new, empty `SmallSet`.
    pub fn new() -> SmallSet<A> {
        SmallSet {
            inner: InnerSmallVec::Stack(SmallVec::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts `elem` into the set if not yet present. Returns `true` if the
    /// set did not have this element present, or `false` if it already had this
    /// element present.
    pub fn insert(&mut self, elem: A::Item) -> bool {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => {
                if elements.contains(&elem) {
                    false
                } else {
                    if elements.len() + 1 <= A::size() {
                        elements.push(elem);
                    } else {
                        let mut ee = HashSet::<A::Item>::with_capacity(elements.len() + 1);
                        while !elements.is_empty() {
                            ee.insert(elements.remove(0));
                        }
                        ee.insert(elem);
                        self.inner = InnerSmallVec::Heap(ee);
                    }
                    true
                }
            }
            InnerSmallVec::Heap(ref mut elements) => elements.insert(elem),
        }
    }

    /// Removes `elem` from the set. Returns `true` if the element was removed,
    /// or `false` if it was not found.
    pub fn remove(&mut self, elem: &A::Item) -> bool {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => {
                if let Some(pos) = elements.iter().position(|e| *e == *elem) {
                    elements.remove(pos);
                    true
                } else {
                    false
                }
            }
            InnerSmallVec::Heap(ref mut elements) => elements.remove(elem),
        }
    }

    /// Tests whether `elem` is present. Returns `true` if it is present, or
    /// `false` if not.
    pub fn contains(&self, elem: &A::Item) -> bool {
        match &self.inner {
            InnerSmallVec::Stack(ref elements) => elements.iter().any(|e| *e == *elem),
            InnerSmallVec::Heap(ref elements) => elements.contains(elem),
        }
    }

    /// Returns an iterator over the set elements. Elements will be returned in
    /// an arbitrary (unsorted) order.
    pub fn iter(&self) -> SmallIter<A> {
        match &self.inner {
            InnerSmallVec::Stack(element) => SmallIter {
                inner: InnerSmallIter::Stack(element.iter()),
            },
            InnerSmallVec::Heap(element) => SmallIter {
                inner: InnerSmallIter::Heap(element.iter()),
            },
        }
    }

    /// Returns the current length of the set.
    pub fn len(&self) -> usize {
        match &self.inner {
            InnerSmallVec::Stack(elements) => elements.len(),
            InnerSmallVec::Heap(elements) => elements.len(),
        }
    }

    /// Clears the set.
    pub fn clear(&mut self) {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => elements.clear(),
            InnerSmallVec::Heap(ref mut elements) => {
                elements.clear();
                self.inner = Default::default();
            }
        }
    }

    //
    pub fn get(&self, value: &A::Item) -> Option<&A::Item> {
        match &self.inner {
            InnerSmallVec::Stack(elements) => elements.iter().find(|x| (value).eq(&x)),
            InnerSmallVec::Heap(elements) => elements.iter().find(|x| (value).eq(&x)),
        }
    }

    pub fn take(&mut self, value: &A::Item) -> Option<A::Item> {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => {
                if let Some(pos) = elements.iter().position(|e| *e == *value) {
                    let result = elements.remove(pos);
                    Some(result)
                } else {
                    None
                }
            }
            InnerSmallVec::Heap(ref mut elements) => elements.take(value),
        }
    }

    // Adds a value to the set, replacing the existing value, if any, that is equal to the given one. Returns the replaced value.
    pub fn replace(&mut self, value: A::Item) -> Option<A::Item> {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => {
                if let Some(pos) = elements.iter().position(|e| *e == value) {
                    let result = elements.remove(pos);
                    elements.insert(pos, value);
                    Some(result)
                } else {
                    None
                }
            }
            InnerSmallVec::Heap(ref mut elements) => elements.replace(value),
        }
    }

    pub fn drain(&mut self) -> SmallDrain<A::Item> {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => {
                // TODO: Clean up this garbage...
                let mut ee = Vec::<A::Item>::with_capacity(elements.len() + 1);
                while !elements.is_empty() {
                    ee.push(elements.remove(0));
                }
                SmallDrain { data: ee, index: 0 }
            }
            InnerSmallVec::Heap(ref mut elements) => {
                let drain = elements.drain().collect::<Vec<A::Item>>();
                SmallDrain {
                    data: drain,
                    index: 0,
                }
            }
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&mut A::Item) -> bool + for<'r> FnMut(&'r <A as smallvec::Array>::Item) -> bool,
    {
        match &mut self.inner {
            InnerSmallVec::Stack(ref mut elements) => elements.retain(f),
            InnerSmallVec::Heap(ref mut elements) => elements.retain(f),
        }
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> SmallIntersection<'a, A::Item> {
        match &self.inner {
            InnerSmallVec::Stack(ref elements) => {
                let result = elements
                    .iter()
                    .filter(|x| other.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                SmallIntersection {
                    data: result,
                    index: 0,
                }
            }

            InnerSmallVec::Heap(ref elements) => {
                let result = elements
                    .iter()
                    .filter(|x| other.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                SmallIntersection {
                    data: result,
                    index: 0,
                }
            }
        }
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> SmallUnion<'a, A::Item> {
        match &self.inner {
            InnerSmallVec::Stack(ref elements) => {
                let mut lhs = elements.iter().collect::<Vec<&'a A::Item>>();
                let mut rhs = other
                    .iter()
                    .filter(|x| !lhs.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                lhs.append(&mut rhs);
                SmallUnion {
                    data: lhs,
                    index: 0,
                }
            }

            InnerSmallVec::Heap(ref elements) => {
                let mut lhs = elements.iter().collect::<Vec<&'a A::Item>>();
                let mut rhs = other
                    .iter()
                    .filter(|x| !lhs.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                lhs.append(&mut rhs);
                SmallUnion {
                    data: rhs,
                    index: 0,
                }
            }
        }
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> SmallDifference<'a, A::Item> {
        match &self.inner {
            InnerSmallVec::Stack(ref elements) => {
                let lhs = elements
                    .iter()
                    .filter(|x| !other.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                SmallDifference {
                    data: lhs,
                    index: 0,
                }
            }

            InnerSmallVec::Heap(ref elements) => {
                let lhs = elements
                    .iter()
                    .filter(|x| !other.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                SmallDifference {
                    data: lhs,
                    index: 0,
                }
            }
        }
    }

    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a Self,
    ) -> SmallSymmetricDifference<'a, A::Item> {
        match &self.inner {
            InnerSmallVec::Stack(ref elements) => {
                let mut lhs = elements
                    .iter()
                    .filter(|x| !other.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                let mut rhs = other
                    .iter()
                    .filter(|x| !elements.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                lhs.append(&mut rhs);
                SmallSymmetricDifference {
                    data: lhs,
                    index: 0,
                }
            }

            InnerSmallVec::Heap(ref elements) => {
                let mut lhs = elements
                    .iter()
                    .filter(|x| other.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                let mut rhs = other
                    .iter()
                    .filter(|x| elements.contains(x))
                    .collect::<Vec<&'a A::Item>>();
                lhs.append(&mut rhs);
                SmallSymmetricDifference {
                    data: lhs,
                    index: 0,
                }
            }
        }
    }
}

pub struct SmallDrain<T> {
    data: Vec<T>,
    index: usize,
}

impl<T> Iterator for SmallDrain<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.data.len() {
            None
        } else {
            let ptr = self.data.as_ptr();
            self.index += 1;
            unsafe { Some(std::ptr::read(ptr.add(self.index - 1))) }
        }
    }
}

pub struct SmallIntersection<'a, T> {
    data: Vec<&'a T>,
    index: usize,
}

impl<'a, T> Iterator for SmallIntersection<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.data.len() {
            None
        } else {
            let ptr = self.data.as_ptr();
            self.index += 1;
            unsafe { Some(std::ptr::read(ptr.add(self.index - 1))) }
        }
    }
}

pub struct SmallUnion<'a, T> {
    data: Vec<&'a T>,
    index: usize,
}

impl<'a, T> Iterator for SmallUnion<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.data.len() {
            None
        } else {
            let ptr = self.data.as_ptr();
            self.index += 1;
            unsafe { Some(std::ptr::read(ptr.add(self.index - 1))) }
        }
    }
}

pub struct SmallDifference<'a, T> {
    data: Vec<&'a T>,
    index: usize,
}

impl<'a, T> Iterator for SmallDifference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.data.len() {
            None
        } else {
            let ptr = self.data.as_ptr();
            self.index += 1;
            unsafe { Some(std::ptr::read(ptr.add(self.index - 1))) }
        }
    }
}

pub struct SmallSymmetricDifference<'a, T> {
    data: Vec<&'a T>,
    index: usize,
}

impl<'a, T> Iterator for SmallSymmetricDifference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.data.len() {
            None
        } else {
            let ptr = self.data.as_ptr();
            self.index += 1;
            unsafe { Some(std::ptr::read(ptr.add(self.index - 1))) }
        }
    }
}

impl<A: Array> Clone for SmallSet<A>
where
    A::Item: PartialEq + Eq + Clone,
{
    fn clone(&self) -> SmallSet<A> {
        SmallSet {
            inner: self.inner.clone(),
        }
    }
}

impl<A: Array> fmt::Debug for SmallSet<A>
where
    A::Item: PartialEq + Eq + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.inner {
            InnerSmallVec::Stack(elements) => write!(f, "{:?}", elements.as_slice()),
            InnerSmallVec::Heap(elements) => write!(f, "{:?}", elements),
        }
    }
}

impl<A: Array> FromIterator<A::Item> for SmallSet<A>
where
    A::Item: PartialEq + Eq + Hash,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A::Item>,
    {
        iter.into_iter().fold(SmallSet::new(), |mut acc, x| {
            acc.insert(x);
            acc
        })
    }
}

pub struct SmallIter<'a, A: Array>
where
    A::Item: PartialEq + Eq + Hash + 'a,
{
    inner: InnerSmallIter<'a, A>,
}

pub enum InnerSmallIter<'a, A: Array>
where
    A::Item: PartialEq + Eq + Hash + 'a,
{
    Stack(std::slice::Iter<'a, A::Item>),
    Heap(std::collections::hash_set::Iter<'a, A::Item>),
}

impl<'a, A: Array> Iterator for SmallIter<'a, A>
where
    A::Item: PartialEq + Eq + Hash + 'a,
{
    type Item = &'a A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            InnerSmallIter::Stack(ref mut iter) => iter.next(),
            InnerSmallIter::Heap(ref mut iter) => iter.next(),
        }
    }
}
