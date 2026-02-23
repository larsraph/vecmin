#![no_std]

pub mod vec;

extern crate alloc;

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

/// Copied from `smallvec` who copied from unstable `slice::range` in `core` to avoid depending on unstable features.
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

#[test]
pub fn test() {
    let a = vec1![1i32, 2, 3];
    let b = vec1![2; 3];

    let a = vecmin![5; [1; 5]];
    let b = vecmin![5; [1, 1, 1, 1, 1]];
    let c = vecmin![1; 5];
    let d = vecmin![1, 2, 3];
    let e = vecmin![1; [1; 0]];
}

#[macro_export]
macro_rules! vec1 {
    () => {
        compile_error!("VecOne needs at least 1 element")
    };
    ($elem:expr; 0) => {
        compile_error!("VecOne needs at least 1 element")
    };
    ($($x:expr),* $(,)?) => {
        $crate::VecOne::new_from_vec(<[_]>::into_vec(::alloc::boxed::Box::new([$($x),+]))).expect("infallible")
    };
    ($elem:expr; $n:expr) => {
        $crate::VecOne::new_from_vec(::alloc::vec::from_elem($elem, $n)).expect("infallible")
    };
}

#[macro_export]
macro_rules! vecmin {
    ($min:expr; [$x:expr; $n:expr]) => {{
        const M: usize = $min;
        $crate::VecMin::<_, M>::new_from_vec(::alloc::vec::from_elem($x, $n)).expect("length of vec must be greater than than minimum required")
    }};
    ($min:expr; [$($x:expr),+ $(,)?]) => {{
        const M: usize = $min;
        $crate::VecMin::<_, M>::new_from_vec(<[_]>::into_vec(::alloc::boxed::Box::new([$($x),+]))).expect("length of vec must be greater than than minimum required")
    }};
    ($x:expr; $n:expr) => {{
        const M: usize = $n;
        $crate::VecMin::<_, M>::new_from_vec(::alloc::vec::from_elem($x, $n)).expect("infallible")
    }};
    ($($x:expr),+ $(,)?) => {{
        const M: usize = <[()]>::len(&[$( { let _ = &$x; () }),+]);
        $crate::VecMin::<_, M>::new_from_vec(<[_]>::into_vec(::alloc::boxed::Box::new([$($x),+]))).expect("infallible")
    }};
}
