use rayon::prelude::*;
use std::ops::Range;

/// A struct which calculates a spiral in distinct coordinates.
pub struct Spiral {
  range: Range<i64>,
}

impl Spiral {
  /// Creates a new Spiral which will produce values as high as `max`.
  pub fn new(max: i64) -> Spiral {
    let top = (max * 2 + 1).pow(2) - 1;
    Spiral { range: (1..top) }
  }

  /// Returns an iterator over the spiral values.
  pub fn iter(self) -> impl Iterator<Item = (i64, i64)> {
    self.range.into_iter().map(|x| Self::pair(x))
  }

  /// Returns a parallel iterator over the spiral values.
  pub fn par_iter(self) -> impl rayon::iter::ParallelIterator<Item = (i64, i64)> {
    self.range.into_par_iter().map(|x| Self::pair(x))
  }

  /// Calculates the next coordinates in the spiral.
  fn pair(n: i64) -> (i64, i64) {
    let s = (((n as f64).sqrt() + 1.0) / 2.0) as i64;
    let l = (n - ((2 * s) - 1).pow(2)) / (s * 2);
    let e = (n - ((2 * s) - 1).pow(2)) - (2 * s * l) - s + 1;

    match l {
      0 => (s, e),
      1 => (-e, s),
      2 => (-s, -e),
      _ => (e, -s),
    }
  }
}
