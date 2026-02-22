// use std::borrow::{Borrow, BorrowMut, Cow};
// use std::iter::repeat_with;
// use std::ops::{Bound, Deref, RangeBounds};
// use std::slice;

// use smallvec::{Array, CollectionAllocErr, SmallVec};

// use crate::MinLenError;

// #[repr(transparent)]
// // #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct SmallVecMin<T: Array, const M: usize>
// // where
// // T::Item: Clone + Debug,
// {
//     vec: SmallVec<T>,
// }

// // --- Custom ---
// impl<T: Array, const M: usize> SmallVecMin<T, M> {
//     /// Returns the minimum length of the vector.
//     #[inline]
//     pub const fn min_len(&self) -> usize {
//         M
//     }

//     /// Returns a slice to the first `M` elements of the vector, which are guaranteed to exist.
//     #[inline]
//     pub fn min_slice(&self) -> &[T::Item; M] {
//         unsafe { &*(self.vec.as_ptr() as *const [T::Item; M]) }
//     }
// }

// /// --- Iterators ---
// impl<T: Array, const M: usize> IntoIterator for SmallVecMin<T, M> {
//     type Item = T::Item;
//     type IntoIter = smallvec::IntoIter<T>;

//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.vec.into_iter()
//     }
// }

// impl<'a, T: Array + 'a, const M: usize> IntoIterator for &'a SmallVecMin<T, M> {
//     type Item = &'a T::Item;
//     type IntoIter = slice::Iter<'a, T::Item>;

//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.vec.iter()
//     }
// }

// impl<'a, T: Array + 'a, const M: usize> IntoIterator for &'a mut SmallVecMin<T, M> {
//     type Item = &'a mut T::Item;
//     type IntoIter = slice::IterMut<'a, T::Item>;

//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.vec.iter_mut()
//     }
// }

// // --- Constructors & Destructors & Conversion ---
// impl<T: Array, const M: usize> SmallVecMin<T, M> {
//     /// Creates a new `VecMin` from a `SmallVec`
//     ///
//     /// # Safety
//     /// - The length of the provided `SmallVec` must be at least `M`.
//     #[inline]
//     pub const unsafe fn new_unchecked(vec: SmallVec<T>) -> Self {
//         Self { vec }
//     }

//     /// Creates a new `VecMin` from a `SmallVec`, returning an error if the length of the provided `SmallVec` is less than `M`.
//     #[inline]
//     pub fn new(vec: SmallVec<T>) -> Result<Self, SmallVec<T>> {
//         if vec.len() >= M {
//             Ok(unsafe { Self::new_unchecked(vec) })
//         } else {
//             Err(vec)
//         }
//     }

//     #[inline]
//     fn _collect(iter: impl IntoIterator<Item = T::Item>, cap: usize) -> Result<Self, SmallVec<T>> {
//         let iter = iter.into_iter();
//         let (hint, _) = iter.size_hint();

//         let mut vec = SmallVec::with_capacity(cap.max(hint));
//         vec.extend(iter);

//         Self::new(vec)
//     }

//     /// Creates a new `VecMin` from an iterator, returning an error if the length of the collected `SmallVec` is less than `M`.
//     #[inline]
//     pub fn collect(iter: impl IntoIterator<Item = T::Item>) -> Result<Self, SmallVec<T>> {
//         Self::_collect(iter, M)
//     }

//     /// Creates a new `VecMin` from an iterator, returning an error if the length of the collected `SmallVec` is less than `M`.
//     /// The provided `extra` capacity is added to the minimum capacity of `M` when collecting the iterator.
//     #[inline]
//     pub fn collect_with_capacity(
//         iter: impl IntoIterator<Item = T::Item>,
//         extra: usize,
//     ) -> Result<Self, SmallVec<T>> {
//         Self::_collect(iter, M + extra)
//     }

//     /// Returns the inner `SmallVec`, consuming the `VecMin`.
//     #[inline]
//     pub fn into_inner(self) -> SmallVec<T> {
//         self.vec
//     }

//     /// See [`SmallVec::into_boxed_slice`].
//     #[inline]
//     pub fn into_boxed_slice(self) -> Box<[T::Item]> {
//         self.vec.into_boxed_slice()
//     }
// }

// impl<T: Array, const M: usize> Default for SmallVecMin<T, M>
// where
//     T::Item: Default,
// {
//     #[inline]
//     fn default() -> Self {
//         Self {
//             vec: repeat_with(T::Item::default).take(M).collect(),
//         }
//     }
// }

// impl<T: Array, const M: usize> TryFrom<SmallVec<T>> for SmallVecMin<T, M> {
//     type Error = SmallVec<T>;

//     #[inline]
//     fn try_from(vec: SmallVec<T>) -> Result<Self, Self::Error> {
//         Self::new(vec)
//     }
// }

// impl<T: Array, const M: usize> From<SmallVecMin<T, M>> for SmallVec<T> {
//     #[inline]
//     fn from(vec_min: SmallVecMin<T, M>) -> Self {
//         vec_min.vec
//     }
// }

// impl<T: Array, const M: usize> TryFrom<Box<[T::Item]>> for SmallVecMin<T, M> {
//     type Error = SmallVec<T>;

//     #[inline]
//     fn try_from(boxed_slice: Box<[T::Item]>) -> Result<Self, Self::Error> {
//         let vec = boxed_slice.into_vec().into();
//         Self::new(vec)
//     }
// }

// impl<T: Array, const M: usize> From<SmallVecMin<T, M>> for Box<[T::Item]> {
//     #[inline]
//     fn from(vec_min: SmallVecMin<T, M>) -> Self {
//         vec_min.into_boxed_slice()
//     }
// }

// impl<T: Array, const M: usize> TryFrom<&[T::Item]> for SmallVecMin<T, M>
// where
//     T::Item: Clone,
// {
//     type Error = SmallVec<T>;

//     #[inline]
//     fn try_from(slice: &[T::Item]) -> Result<Self, Self::Error> {
//         Self::new(slice.to_vec().into())
//     }
// }

// impl<T: Array, const M: usize> TryFrom<&mut [T::Item]> for SmallVecMin<T, M>
// where
//     T::Item: Clone,
// {
//     type Error = SmallVec<T>;

//     #[inline]
//     fn try_from(slice: &mut [T::Item]) -> Result<Self, Self::Error> {
//         Self::new(slice.to_vec().into())
//     }
// }

// impl<T: Array, const M: usize> TryFrom<Cow<'_, [T::Item]>> for SmallVecMin<T, M>
// where
//     T::Item: Clone,
// {
//     type Error = SmallVec<T>;

//     #[inline]
//     fn try_from(cow: Cow<'_, [T::Item]>) -> Result<Self, Self::Error> {
//         Self::new(cow.into_owned().into())
//     }
// }

// // impl<T: Array, const M: usize> TryFrom<T> for SmallVecMin<T, M> {
// //     type Error = SmallVec<T>;

// //     #[inline]
// //     fn try_from(array: T) -> Result<Self, Self::Error> {
// //         Self::new(array.into())
// //     }
// // }

// // --- Immutable Len Access ---
// impl<T: Array, const M: usize> Deref for SmallVecMin<T, M> {
//     type Target = SmallVec<T>;

//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         &self.vec
//     }
// }

// // --- Mutable Len Access ---

// // -- Not Len Decreasing --

// // - Cap -
// impl<T: Array, const M: usize> SmallVecMin<T, M> {
//     /// See [`SmallVec::reserve`].
//     #[inline]
//     pub fn reserve(&mut self, additional: usize) {
//         self.vec.reserve(additional);
//     }

//     /// See [`SmallVec::reserve_exact`].
//     #[inline]
//     pub fn reserve_exact(&mut self, additional: usize) {
//         self.vec.reserve_exact(additional);
//     }

//     /// See [`SmallVec::try_reserve`].
//     #[inline]
//     pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
//         self.vec.try_reserve(additional)
//     }

//     /// See [`SmallVec::try_reserve_exact`].
//     #[inline]
//     pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
//         self.vec.try_reserve_exact(additional)
//     }

//     /// See [`SmallVec::shrink_to_fit`].
//     #[inline]
//     pub fn shrink_to_fit(&mut self) {
//         self.vec.shrink_to_fit();
//     }

//     // /// See [`SmallVec::shrink_to`].
//     // #[inline]
//     // pub fn shrink_to(&mut self, min_capacity: usize) {
//     //     self.vec.shrink_to(min_capacity);
//     // }
// }

// // - View -
// impl<T: Array, const M: usize> SmallVecMin<T, M> {
//     /// See [`SmallVec::as_mut_slice`].
//     #[inline]
//     pub fn as_mut_slice(&mut self) -> &mut [T::Item] {
//         self.vec.as_mut_slice()
//     }

//     /// See [`SmallVec::as_mut_ptr`].
//     #[inline]
//     pub fn as_mut_ptr(&mut self) -> *mut T::Item {
//         self.vec.as_mut_ptr()
//     }

//     // /// See [`SmallVec::spare_capacity_mut`].
//     // #[inline]
//     // pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T::Item>] {
//     //     self.vec.spare_capacity_mut()
//     // }
// }

// impl<T: Array, const M: usize> AsRef<[T::Item]> for SmallVecMin<T, M> {
//     #[inline]
//     fn as_ref(&self) -> &[T::Item] {
//         &self.vec
//     }
// }

// impl<T: Array, const M: usize> AsMut<[T::Item]> for SmallVecMin<T, M> {
//     #[inline]
//     fn as_mut(&mut self) -> &mut [T::Item] {
//         &mut self.vec
//     }
// }

// impl<T: Array, const M: usize> Borrow<[T::Item]> for SmallVecMin<T, M> {
//     #[inline]
//     fn borrow(&self) -> &[T::Item] {
//         &self.vec
//     }
// }

// impl<T: Array, const M: usize> BorrowMut<[T::Item]> for SmallVecMin<T, M> {
//     #[inline]
//     fn borrow_mut(&mut self) -> &mut [T::Item] {
//         &mut self.vec
//     }
// }

// // - Increasing len -
// impl<T: Array, const M: usize> SmallVecMin<T, M> {
//     /// See [`SmallVec::push`].
//     #[inline]
//     pub fn push(&mut self, item: T::Item) {
//         self.vec.push(item);
//     }

//     /// See [`SmallVec::insert`].
//     #[inline]
//     pub fn insert(&mut self, index: usize, element: T::Item) {
//         self.vec.insert(index, element);
//     }

//     /// See [`SmallVec::append`].
//     #[inline]
//     pub fn append(&mut self, other: &mut SmallVec<T>) {
//         self.vec.append(other);
//     }

//     /// See [`SmallVec::extend_from_slice`].
//     #[inline]
//     pub fn extend_from_slice(&mut self, other: &[T::Item])
//     where
//         T::Item: Copy,
//     {
//         self.vec.extend_from_slice(other);
//     }

//     // /// See [`SmallVec::extend_from_within`].
//     // #[inline]
//     // pub fn extend_from_within<R>(&mut self, range: R)
//     // where
//     //     R: std::ops::RangeBounds<usize>,
//     //     T::Item: Copy,
//     // {
//     //     self.vec.extend_from_within(range);
//     // }
// }

// impl<T: Array, const M: usize> Extend<T::Item> for SmallVecMin<T, M> {
//     #[inline]
//     fn extend<I: IntoIterator<Item = T::Item>>(&mut self, iter: I) {
//         self.vec.extend(iter);
//     }
// }

// // impl<'a, T: Array, const M: usize> Extend<&'a T::Item> for SmallVecMin<T, M>
// // where
// //     T::Item: Copy + 'a,
// // {
// //     #[inline]
// //     fn extend<I: IntoIterator<Item = &'a T::Item>>(&mut self, iter: I) {
// //         self.vec.extend(iter);
// //     }
// // }

// // -- Len Decreasing --
// impl<T: Array, const M: usize> SmallVecMin<T, M> {
//     /// See [`SmallVec::pop`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[inline]
//     #[must_use]
//     pub fn pop(&mut self) -> Result<Option<T::Item>, MinLenError> {
//         if self.vec.len() > M {
//             Ok(self.vec.pop())
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     /// See [`SmallVec::pop`]. Pops an element from the vector if the length of the vector is greater than `M`, otherwise does nothing and returns `None`.
//     #[inline]
//     pub fn pop_to_min(&mut self) -> Option<T::Item> {
//         if self.vec.len() > M {
//             self.vec.pop()
//         } else {
//             None
//         }
//     }

//     /// See [`SmallVec::remove`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[inline]
//     #[must_use]
//     pub fn remove(&mut self, index: usize) -> Result<T::Item, MinLenError> {
//         if self.vec.len() > M {
//             Ok(self.vec.remove(index))
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     /// See [`SmallVec::swap_remove`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[inline]
//     #[must_use]
//     pub fn swap_remove(&mut self, index: usize) -> Result<T::Item, MinLenError> {
//         if self.vec.len() > M {
//             Ok(self.vec.swap_remove(index))
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     /// See [`SmallVec::truncate`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[inline]
//     #[must_use]
//     pub fn truncate(&mut self, len: usize) -> Result<(), MinLenError> {
//         if len >= M {
//             Ok(self.vec.truncate(len))
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     /// See [`SmallVec::truncate`]. Truncates the vector to `len` if `len` is greater than or equal to `M`, otherwise truncates the vector to `M`.
//     #[inline]
//     pub fn truncate_or_min(&mut self, len: usize) {
//         self.vec.truncate(len.max(M))
//     }

//     /// See [`SmallVec::truncate`]. Truncates the vector to `M`.
//     #[inline]
//     pub fn truncate_to_min(&mut self) {
//         self.vec.truncate(M);
//     }

//     /// See [`SmallVec::resize`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[inline]
//     #[must_use]
//     pub fn resize(&mut self, new_len: usize, value: T::Item) -> Result<(), MinLenError>
//     where
//         T::Item: Clone,
//     {
//         if new_len >= M {
//             Ok(self.vec.resize(new_len, value))
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     /// See [`SmallVec::resize`]. Resizes the vector to `new_len` if `new_len` is greater than or equal to `M`, otherwise resizes the vector to `M`.
//     #[inline]
//     #[must_use]
//     pub fn resize_or_min(&mut self, new_len: usize, value: T::Item)
//     where
//         T::Item: Clone,
//     {
//         self.vec.resize(new_len.max(M), value);
//     }

//     /// See [`SmallVec::resize_with`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[inline]
//     #[must_use]
//     pub fn resize_with<F>(&mut self, new_len: usize, generator: F) -> Result<(), MinLenError>
//     where
//         F: FnMut() -> T::Item,
//     {
//         if new_len >= M {
//             Ok(self.vec.resize_with(new_len, generator))
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     /// See [`SmallVec::resize_with`]. Resizes the vector to `new_len` if `new_len` is greater than or equal to `M`, otherwise resizes the vector to `M`.
//     #[inline]
//     #[must_use]
//     pub fn resize_or_min_with<F>(&mut self, new_len: usize, generator: F)
//     where
//         F: FnMut() -> T::Item,
//     {
//         self.vec.resize_with(new_len.max(M), generator);
//     }

//     /// See [`SmallVec::drain`]. Returns an error if the operation would reduce the length of the vector below `M`.
//     #[must_use]
//     pub fn drain<R>(&mut self, range: R) -> Result<smallvec::Drain<'_, T>, MinLenError>
//     where
//         R: RangeBounds<usize>,
//     {
//         let drain_len = range_len(&range, self.vec.len());
//         let final_len = self.vec.len() - drain_len;

//         if final_len >= M {
//             Ok(self.vec.drain(range))
//         } else {
//             Err(MinLenError::BelowMinimum)
//         }
//     }

//     // /// See [`SmallVec::splice`]. Drains the specified range if draining it would not reduce the length of the vector below `M`, otherwise does nothing and returns an empty iterator.
//     // #[must_use]
//     // pub fn splice<R, I>(
//     //     &mut self,
//     //     range: R,
//     //     replace_with: I,
//     // ) -> Result<smallvec::Splice<'_, I::IntoIter>, MinLenError>
//     // where
//     //     R: RangeBounds<usize>,
//     //     I: IntoIterator<Item = T::Item, IntoIter: ExactSizeIterator>,
//     // {
//     //     let replace_with = replace_with.into_iter();

//     //     let gain = replace_with.len();
//     //     let loss = range_len(&range, self.vec.len());
//     //     let final_len = self.vec.len() + gain - loss;

//     //     if final_len >= M {
//     //         Ok(self.vec.splice(range, replace_with))
//     //     } else {
//     //         Err(MinLenError::BelowMinimum)
//     //     }
//     // }
// }

// #[inline]
// fn range_len(range: &impl RangeBounds<usize>, max: usize) -> usize {
//     let start = match range.start_bound() {
//         Bound::Included(&n) => n,
//         Bound::Excluded(&n) => n.saturating_add(1),
//         Bound::Unbounded => 0,
//     };
//     let end = match range.end_bound() {
//         Bound::Included(&n) => n.saturating_add(1),
//         Bound::Excluded(&n) => n,
//         Bound::Unbounded => max,
//     };

//     end.saturating_sub(start)
// }
