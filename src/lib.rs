#![no_std]

pub mod vec;

extern crate alloc;

#[doc(hidden)]
pub extern crate alloc as __alloc;

use core::error::Error;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::{Bound, Range, RangeBounds, RangeTo};

pub use vec::{VecMin, VecOne};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifyError<const M: usize>;

impl<const M: usize> Display for ModifyError<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "operation would reduce length below minimum required {}",
            M
        )
    }
}

impl<const M: usize> Error for ModifyError<M> {}

#[inline]
#[track_caller]
fn slice_range<R>(range: &R, bounds: RangeTo<usize>) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let len = bounds.end;

    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start
            .checked_add(1)
            .expect("attempted to index slice from after maximum usize"),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(&end) => end
            .checked_add(1)
            .expect("attempted to index slice up to maximum usize"),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    assert!(
        start <= end,
        "slice index starts at {start} but ends at {end}"
    );
    assert!(
        end <= len,
        "range end index {end} out of range for slice of length {len}"
    );

    Range { start, end }
}

/// Creates a [`VecOne`] containing the arguments.
///
/// `vecone!` allows `VecOne`s to be defined with similar syntax to `vec!`, but with a minimum length of 1.
/// The length must be constant, for non-constant length use checked constructors.
#[macro_export]
macro_rules! vecone {
    ($($x:expr),+ $(,)?) => {
        $crate::vecmin![1; [$($x),+]]
    };
    ($elem:expr; $n:expr) => {
        $crate::vecmin![1; [$elem; $n]]
    };
}

/// Creates a [`VecMin`] containing the arguments.
///
/// There is a minimum length argument preceding the list, if not included the minimum is inferred as the length.
/// The length must be constant, for non-constant length use checked constructors.
///
/// - Create a [`VecMin`] with a fixed minimum length:
/// ```
/// use vec_min::vecmin;
///
/// let v = vecmin![5; [1; 6]];
/// assert!(!v.is_minimum());
/// assert_eq!(v.minimum(), 5);
///
/// let v = vecmin![2; [1, 2, 3]];
/// assert!(!v.is_minimum());
/// assert_eq!(v.minimum(), 2);
/// ```
///
/// - Create a [`VecMin`] with a minimum length inferred from the number of elements:
/// ```
/// use vec_min::vecmin;
///
/// let v = vecmin![1; 6];
/// assert!(v.is_minimum());
/// assert_eq!(v.minimum(), 6);
///
/// let v = vecmin![1, 2, 3];
/// assert!(v.is_minimum());
/// assert_eq!(v.minimum(), 3);
/// ```
#[macro_export]
macro_rules! vecmin {
    ($min:expr; [$x:expr; $n:expr]) => {{
        let _: [(); $n - $min];
        unsafe { $crate::VecMin::<_, $min>::from_vec_unchecked($crate::__alloc::vec![$x; $n]) }
    }};
    ($min:expr; [$($x:expr),+ $(,)?]) => {{
        const N: usize = <[()]>::len(&[$( { let _ = &$x; () }),+]);
        let _: [(); N - $min];
        unsafe { $crate::VecMin::<_, $min>::from_vec_unchecked($crate::__alloc::vec![$($x),+]) }
    }};
    ($x:expr; $n:expr) => {
        $crate::VecMin::from_array([$x; $n])
    };
    ($($x:expr),+ $(,)?) => {
        $crate::VecMin::from_array([$($x),+])
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn slice_range() {
        use super::slice_range;

        assert_eq!(slice_range(&(..), ..5), 0..5);
        assert_eq!(slice_range(&(1..), ..5), 1..5);
        assert_eq!(slice_range(&(..3), ..5), 0..3);
        assert_eq!(slice_range(&(1..3), ..5), 1..3);
        assert_eq!(slice_range(&(1..=3), ..5), 1..4);
    }

    #[test]
    fn vecone() {
        let v = vecone![1, 1, 1];

        assert_eq!(v.minimum(), 1);
        assert_eq!(v.len(), 3);

        let v = vecone![1; 3];

        assert_eq!(v.minimum(), 1);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn vecmin_explicit() {
        let v = vecmin![2; [1, 1, 1]];

        assert_eq!(v.minimum(), 2);
        assert_eq!(v.len(), 3);

        let v = vecmin![2; [1; 3]];

        assert_eq!(v.minimum(), 2);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn vecmin_implicit() {
        let v = vecmin![1, 1, 1];

        assert_eq!(v.minimum(), 3);
        assert_eq!(v.len(), 3);

        let v = vecmin![1; 3];

        assert_eq!(v.minimum(), 3);
        assert_eq!(v.len(), 3);
    }

    // ---- compile errors ----
    // fn lt_min() {
    //     let v = vecone![];
    //     let v = vecone![1; 0];
    //     let v = vecmin![5; [1, 1, 1, 1]];
    //     let v = vecmin![5; [1; 4]];
    // }

    // fn nonconst() {
    //     let n = 0;
    //     let v = vecone![2; n];
    //     let v = vecmin![2; [1; n]];
    // }
}
