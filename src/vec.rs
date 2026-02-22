use std::borrow::{Borrow, BorrowMut, Cow};
use std::collections::TryReserveError;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::iter::repeat_with;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::{slice, vec};

use crate::{ModifyError, slice_range};

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecMin<T, const M: usize> {
    vec: Vec<T>,
}

impl<T: Debug, const M: usize> Debug for VecMin<T, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("VecMin")
            .field("min", &M)
            .field("vec", &self.vec)
            .finish()
    }
}

// --- Custom ---
impl<T, const M: usize> VecMin<T, M> {
    /// Returns the minimum length of the vector.
    #[inline]
    pub const fn min_len(&self) -> usize {
        M
    }

    /// Returns a slice to the first `M` elements of the vector, which are guaranteed to exist.
    #[inline]
    pub const fn min_slice(&self) -> &[T; M] {
        self.split_at_min().0
    }

    /// Returns a mutable slice to the first `M` elements of the vector, which are guaranteed to exist.
    #[inline]
    pub const fn min_slice_mut(&mut self) -> &mut [T; M] {
        self.split_at_min_mut().0
    }

    /// Returns a tuple of a slice to the first `M` elements of the vector, which are guaranteed to exist, and a slice to the remaining elements of the vector.
    #[inline]
    pub const fn split_at_min(&self) -> (&[T; M], &[T]) {
        debug_assert!(self.vec.len() >= M);

        // Safety: Structural invariant that the `Vec` has a minimum length of `M`.
        let min = unsafe { &*self.vec.as_ptr().cast() };

        // Safety: Structural invariant that the `Vec` has a minimum length of `M`.
        let extra = unsafe {
            let data = self.vec.as_ptr().add(M);
            let len = self.vec.len() - M;
            slice::from_raw_parts(data, len)
        };

        (min, extra)
    }

    /// Returns a tuple of a mutable slice to the first `M` elements of the vector, which are guaranteed to exist, and a mutable slice to the remaining elements of the vector.
    #[inline]
    pub const fn split_at_min_mut(&mut self) -> (&mut [T; M], &mut [T]) {
        debug_assert!(self.vec.len() >= M);

        // Safety: Structural invariant that the `Vec` has a minimum length of `M`.
        let min = unsafe { &mut *self.vec.as_mut_ptr().cast() };

        // Safety: Structural invariant that the `Vec` has a minimum length of `M`.
        let extra = unsafe {
            let data = self.vec.as_mut_ptr().add(M);
            let len = self.vec.len() - M;
            slice::from_raw_parts_mut(data, len)
        };

        (min, extra)
    }
}

// --- Constructors, Convertors, and Destructors ---
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructError<T, const M: usize>(pub Vec<T>);

impl<T: Debug, const M: usize> Display for ConstructError<T, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Length {} of {:?} is less than the minimum {}",
            self.0.len(),
            self.0,
            M
        )
    }
}

impl<T: Debug, const M: usize> Error for ConstructError<T, M> {}

impl<T, const M: usize> VecMin<T, M> {
    /// Creates a new `VecMin` from a `Vec`
    ///
    /// # Safety
    /// - The length of the `Vec` must be at least `M`.
    #[inline]
    pub const unsafe fn new_unchecked(vec: Vec<T>) -> Self {
        Self { vec }
    }

    /// Creates a new `VecMin` from a `Vec`, returning an error if the length of the provided `Vec` is less than `M`.
    #[inline]
    pub const fn const_new(vec: Vec<T>) -> Result<Self, ConstructError<T, M>> {
        if vec.len() >= M {
            // Safety: We just checked that the length was at least `M`.
            Ok(unsafe { Self::new_unchecked(vec) })
        } else {
            Err(ConstructError(vec))
        }
    }

    /// Creates a new `VecMin` from any type that can be converted into a `Vec`, returning an error if the length of the provided `Vec` is less than `M`.
    #[inline]
    pub fn new(vec: impl Into<Vec<T>>) -> Result<Self, ConstructError<T, M>> {
        let vec = vec.into();
        Self::const_new(vec)
    }

    /// Creates a new `VecMin` from an iterator, returning an error if the length of the collected `Vec` is less than `M`.
    #[inline]
    pub fn collect(iter: impl IntoIterator<Item = T>) -> Result<Self, ConstructError<T, M>> {
        let iter = iter.into_iter();
        let (hint, _) = iter.size_hint();

        Self::collect_with_capacity(iter, hint.max(M))
    }

    /// Creates a new `VecMin` from an iterator, returning an error if the length of the collected `Vec` is less than `M`.
    /// The provided `capacity` is preallocated into the `Vec`.
    #[inline]
    pub fn collect_with_capacity(
        iter: impl IntoIterator<Item = T>,
        capacity: usize,
    ) -> Result<Self, ConstructError<T, M>> {
        let mut vec = Vec::with_capacity(capacity);
        vec.extend(iter);

        Self::new(vec)
    }

    /// Returns the inner `Vec`, consuming the `VecMin`.
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.vec
    }

    /// See [`Vec::into_boxed_slice`].
    #[inline]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.vec.into_boxed_slice()
    }

    /// See [`Vec::leak`].
    #[inline]
    pub fn leak(self) -> &'static mut [T] {
        self.vec.leak()
    }
}

impl<T: Default, const M: usize> Default for VecMin<T, M> {
    #[inline]
    fn default() -> Self {
        Self {
            vec: repeat_with(T::default).take(M).collect(),
        }
    }
}

impl<T, const M: usize> TryFrom<Vec<T>> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(vec)
    }
}

impl<T, const M: usize> From<VecMin<T, M>> for Vec<T> {
    #[inline]
    fn from(vec_min: VecMin<T, M>) -> Self {
        vec_min.vec
    }
}

impl<T, const M: usize> TryFrom<Box<[T]>> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(boxed_slice: Box<[T]>) -> Result<Self, Self::Error> {
        Self::new(boxed_slice)
    }
}

impl<T, const M: usize> From<VecMin<T, M>> for Box<[T]> {
    #[inline]
    fn from(vec_min: VecMin<T, M>) -> Self {
        vec_min.vec.into_boxed_slice()
    }
}

impl<T: Clone, const M: usize> TryFrom<&[T]> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(slice: &[T]) -> Result<Self, Self::Error> {
        Self::new(slice)
    }
}

impl<T: Clone, const M: usize> TryFrom<&mut [T]> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(slice: &mut [T]) -> Result<Self, Self::Error> {
        Self::new(slice)
    }
}

impl<T: Clone, const M: usize> TryFrom<Cow<'_, [T]>> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(cow: Cow<'_, [T]>) -> Result<Self, Self::Error> {
        Self::new(cow)
    }
}

impl<T, const N: usize, const M: usize> TryFrom<[T; N]> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(array: [T; N]) -> Result<Self, Self::Error> {
        Self::new(array)
    }
}

impl<T: Clone, const N: usize, const M: usize> TryFrom<&[T; N]> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(array: &[T; N]) -> Result<Self, Self::Error> {
        Self::new(array)
    }
}

impl<T: Clone, const N: usize, const M: usize> TryFrom<&mut [T; N]> for VecMin<T, M> {
    type Error = ConstructError<T, M>;

    #[inline]
    fn try_from(array: &mut [T; N]) -> Result<Self, Self::Error> {
        Self::new(array)
    }
}

impl<T, const N: usize, const M: usize> TryFrom<VecMin<T, M>> for [T; N] {
    type Error = VecMin<T, M>;

    #[inline]
    fn try_from(vec_min: VecMin<T, M>) -> Result<[T; N], Self::Error> {
        // Safety: We obtained the original `Vec` from a valid `VecMin`.
        vec_min
            .vec
            .try_into()
            .map_err(|vec| unsafe { VecMin::new_unchecked(vec) })
    }
}

// --- View ---
impl<T, const M: usize> VecMin<T, M> {
    #[inline]
    /// See [`Vec::as_slice`].
    pub const fn as_slice(&self) -> &[T] {
        self.vec.as_slice()
    }

    /// See [`Vec::as_mut_slice`].
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        self.vec.as_mut_slice()
    }

    #[inline]
    /// See [`Vec::as_ptr`].
    pub const fn as_ptr(&self) -> *const T {
        self.vec.as_ptr()
    }

    /// See [`Vec::as_mut_ptr`].
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.vec.as_mut_ptr()
    }

    /// See [`Vec::spare_capacity_mut`].
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.vec.spare_capacity_mut()
    }
}

impl<T, const M: usize> Deref for VecMin<T, M> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.vec.deref()
    }
}

impl<T, const M: usize> DerefMut for VecMin<T, M> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vec.deref_mut()
    }
}

impl<T, const M: usize> AsRef<[T]> for VecMin<T, M> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.vec.as_ref()
    }
}

impl<T, const M: usize> AsMut<[T]> for VecMin<T, M> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.vec.as_mut()
    }
}

impl<T, const M: usize> Borrow<[T]> for VecMin<T, M> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self.vec.borrow()
    }
}

impl<T, const M: usize> BorrowMut<[T]> for VecMin<T, M> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.vec.borrow_mut()
    }
}

// --- Iterators ---
impl<T, const M: usize> IntoIterator for VecMin<T, M> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, T: 'a, const M: usize> IntoIterator for &'a VecMin<T, M> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.vec.as_slice().iter()
    }
}

impl<'a, T: 'a, const M: usize> IntoIterator for &'a mut VecMin<T, M> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.vec.as_mut_slice().iter_mut()
    }
}

// --- Immutable Access ---
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::capacity`].
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// See [`Vec::len`].
    #[inline]
    pub const fn len(&self) -> usize {
        self.vec.len()
    }
}

// --- Mutable Access ---

// -- Not Len Decreasing --

// - Capacity -
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::reserve`].
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }

    /// See [`Vec::reserve_exact`].
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional);
    }

    /// See [`Vec::try_reserve`].
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.vec.try_reserve(additional)
    }

    /// See [`Vec::try_reserve_exact`].
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.vec.try_reserve_exact(additional)
    }

    /// See [`Vec::shrink_to_fit`].
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit();
    }

    /// See [`Vec::shrink_to`].
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.vec.shrink_to(min_capacity);
    }
}

// - Growth -
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::push`].
    #[inline]
    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }

    /// See [`Vec::insert`].
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.vec.insert(index, element);
    }

    /// See [`Vec::append`].
    #[inline]
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec.append(other);
    }

    /// See [`Vec::extend_from_slice`].
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.vec.extend_from_slice(other);
    }

    /// See [`Vec::extend_from_within`].
    #[inline]
    pub fn extend_from_within<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
        T: Clone,
    {
        self.vec.extend_from_within(range);
    }
}

impl<T, const M: usize> Extend<T> for VecMin<T, M> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.vec.extend(iter);
    }
}

impl<'a, T: Copy, const M: usize> Extend<&'a T> for VecMin<T, M> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.vec.extend(iter);
    }
}

// -- Len Decreasing --
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::pop`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn pop(&mut self) -> Result<Option<T>, ModifyError<M>> {
        if self.vec.len() > M {
            Ok(self.vec.pop())
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::pop`]. Pops an element from the vector if the length of the vector is greater than `M`, otherwise does nothing and returns `None`.
    #[inline]
    pub fn pop_to_min(&mut self) -> Option<T> {
        if self.vec.len() > M {
            self.vec.pop()
        } else {
            None
        }
    }

    /// See [`Vec::pop`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// Safety:
    /// - The length of the vector must be greater than `M` such that it doesn't decrease below `M`.
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> Option<T> {
        self.vec.pop()
    }

    /// See [`Vec::pop_if`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    pub fn pop_if(
        &mut self,
        pred: impl FnOnce(&mut T) -> bool,
    ) -> Result<Option<T>, ModifyError<M>> {
        if self.vec.len() > M {
            Ok(self.vec.pop_if(pred))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::pop_if`]. Pops an element from the vector if the length of the vector is greater than `M` and the provided predicate returns `true`, otherwise does nothing and returns `None`.
    #[inline]
    pub fn pop_to_min_if(&mut self, pred: impl FnOnce(&mut T) -> bool) -> Option<T> {
        if self.vec.len() > M {
            self.vec.pop_if(pred)
        } else {
            None
        }
    }

    /// See [`Vec::pop_if`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The length of the vector must be greater than `M` such that it doesn't decrease below `M`.
    #[inline]
    pub unsafe fn pop_if_unchecked(&mut self, pred: impl FnOnce(&mut T) -> bool) -> Option<T> {
        self.vec.pop_if(pred)
    }

    /// See [`Vec::remove`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn remove(&mut self, index: usize) -> Result<T, ModifyError<M>> {
        if self.vec.len() > M {
            Ok(self.vec.remove(index))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::remove`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The length of the vector must be greater than `M` such that it doesn't decrease below `M`.
    #[inline]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        self.vec.remove(index)
    }

    /// See [`Vec::swap_remove`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn swap_remove(&mut self, index: usize) -> Result<T, ModifyError<M>> {
        if self.vec.len() > M {
            Ok(self.vec.swap_remove(index))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::swap_remove`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The length of the vector must be greater than `M` such that it doesn't decrease below `M`.
    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        self.vec.swap_remove(index)
    }

    /// See [`Vec::truncate`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn truncate(&mut self, len: usize) -> Result<(), ModifyError<M>> {
        if len >= M {
            Ok(self.vec.truncate(len))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::truncate`]. Truncates the vector to `len` if `len` is greater than or equal to `M`, otherwise truncates the vector to `M`.
    #[inline]
    pub fn truncate_or_min(&mut self, len: usize) {
        self.vec.truncate(len.max(M))
    }

    /// See [`Vec::truncate`]. Truncates the vector to `M`.
    #[inline]
    pub fn truncate_to_min(&mut self) {
        self.vec.truncate(M);
    }

    /// See [`Vec::truncate`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The target `len` must be greater than or equal to `M`.
    #[inline]
    pub unsafe fn truncate_unchecked(&mut self, len: usize) {
        self.vec.truncate(len)
    }

    /// See [`Vec::resize`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn resize(&mut self, new_len: usize, value: T) -> Result<(), ModifyError<M>>
    where
        T: Clone,
    {
        if new_len >= M {
            Ok(self.vec.resize(new_len, value))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::resize`]. Resizes the vector to `new_len` if `new_len` is greater than or equal to `M`, otherwise resizes the vector to `M`.
    #[inline]
    pub fn resize_or_min(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.vec.resize(new_len.max(M), value);
    }

    /// See [`Vec::resize`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The target `new_len` must be greater than or equal to `M`.
    #[inline]
    pub unsafe fn resize_unchecked(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.vec.resize(new_len, value)
    }

    /// See [`Vec::resize_with`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn resize_with<F>(&mut self, new_len: usize, generator: F) -> Result<(), ModifyError<M>>
    where
        F: FnMut() -> T,
    {
        if new_len >= M {
            Ok(self.vec.resize_with(new_len, generator))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::resize_with`]. Resizes the vector to `new_len` if `new_len` is greater than or equal to `M`, otherwise resizes the vector to `M`.
    #[inline]
    pub fn resize_or_min_with<F>(&mut self, new_len: usize, generator: F)
    where
        F: FnMut() -> T,
    {
        self.vec.resize_with(new_len.max(M), generator);
    }

    /// See [`Vec::resize_with`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The target `new_len` must be greater than or equal to `M`.
    #[inline]
    pub unsafe fn resize_with_unchecked<F>(&mut self, new_len: usize, generator: F)
    where
        F: FnMut() -> T,
    {
        self.vec.resize_with(new_len, generator)
    }

    /// See [`Vec::drain`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[must_use]
    pub fn drain<R>(&mut self, range: R) -> Result<vec::Drain<'_, T>, ModifyError<M>>
    where
        R: RangeBounds<usize>,
    {
        let drain_len = slice_range(&range, ..self.vec.len()).len();
        let final_len = self.vec.len() - drain_len;

        if final_len >= M {
            Ok(self.vec.drain(range))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::drain`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The length of the vector after draining must be greater than or equal to `M`.
    #[inline]
    pub unsafe fn drain_unchecked<R>(&mut self, range: R) -> vec::Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        self.vec.drain(range)
    }

    /// See [`Vec::splice`]. Returns an error if the operation would reduce the length of the vector below `M`.
    ///
    /// Requires `replace_with` to become an ExactSizeIterator unlike [`Vec::splice`]
    #[must_use]
    pub fn splice<R, I>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> Result<vec::Splice<'_, I::IntoIter>, ModifyError<M>>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T, IntoIter: ExactSizeIterator>,
    {
        let replace_with = replace_with.into_iter();

        let gain = replace_with.len();
        let loss = slice_range(&range, ..self.vec.len()).len();
        let final_len = self.vec.len() + gain - loss;

        if final_len >= M {
            Ok(self.vec.splice(range, replace_with))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::split_off`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Result<Vec<T>, ModifyError<M>> {
        if at >= M {
            Ok(self.vec.split_off(at))
        } else {
            Err(ModifyError)
        }
    }

    /// See [`Vec::split_off`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    #[inline]
    pub unsafe fn split_off_unchecked(&mut self, at: usize) -> Vec<T> {
        self.vec.split_off(at)
    }

    /// See [`Vec::splice`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// # Safety
    /// - The length of the vector after splicing must be greater than or equal to `M`.
    #[inline]
    pub unsafe fn splice_unchecked<R, I>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> vec::Splice<'_, I::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        self.vec.splice(range, replace_with)
    }

    /// See [`Vec::retain`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// There is no checked version of the method because the final length is impossible to know.
    #[inline]
    pub unsafe fn retain_unchecked<F>(&mut self, pred: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.vec.retain(pred);
    }

    /// See [`Vec::retain_mut`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// There is no checked version of the method because the final length is impossible to know.
    #[inline]
    pub unsafe fn retain_mut_unchecked<F>(&mut self, pred: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.vec.retain_mut(pred);
    }

    /// See [`Vec::dedup`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// There is no checked version of the method because the final length is impossible to know.
    #[inline]
    pub unsafe fn dedup_unchecked(&mut self)
    where
        T: PartialEq,
    {
        self.vec.dedup();
    }

    /// See [`Vec::dedup_by`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// There is no checked version of the method because the final length is impossible to know.
    #[inline]
    pub unsafe fn dedup_by_unchecked<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.vec.dedup_by(same_bucket);
    }

    /// See [`Vec::dedup_by_key`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// There is no checked version of the method because the final length is impossible to know.
    #[inline]
    pub unsafe fn dedup_by_key_unchecked<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.vec.dedup_by_key(key);
    }

    /// See [`Vec::extract_if`]. Doesn't check if the operation would reduce the length of the vector below `M`.
    ///
    /// There is no checked version of the method because the final length is impossible to know.
    #[inline]
    pub unsafe fn extract_if_unchecked<F, R>(
        &mut self,
        range: R,
        filter: F,
    ) -> vec::ExtractIf<'_, T, F>
    where
        R: RangeBounds<usize>,
        F: FnMut(&mut T) -> bool,
    {
        self.vec.extract_if(range, filter)
    }
}
