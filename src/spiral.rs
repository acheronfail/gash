use rayon::prelude::*;
use std::ops::Range;

/// A struct which calculates a spiral in distinct coordinates.
pub struct Spiral {
    range: Range<i64>,
}

impl Spiral {
    /// Creates a new Spiral which will produce values as high as `max`.
    pub fn new(max: impl Into<i64>) -> Spiral {
        let max = max.into();
        let top = (max * 2 + 1).pow(2) - 1;
        Spiral { range: (1..top) }
    }

    /// Returns an iterator over the spiral values.
    pub fn iter(self) -> impl Iterator<Item = (i64, i64)> {
        self.range.map(Self::pair)
    }

    /// Returns a parallel iterator over the spiral values.
    pub fn par_iter(self) -> impl ParallelIterator<Item = (i64, i64)> {
        self.range.into_par_iter().map(Self::pair)
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

#[cfg(test)]
mod tests {
    use crate::Spiral;

    #[test]
    fn it_works() {
        let spiral = Spiral::new(2);
        let values = spiral.iter().collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (2, 0),
                (2, 1),
                (2, 2),
                (1, 2),
                (0, 2),
                (-1, 2),
                (-2, 2),
                (-2, 1),
                (-2, 0),
                (-2, -1),
                (-2, -2),
                (-1, -2),
                (0, -2),
                (1, -2)
            ],
        );
    }

    #[test]
    fn it_works_parallel() {
        use rayon::iter::ParallelIterator;

        let spiral = Spiral::new(2);
        let values = spiral.par_iter().collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (2, 0),
                (2, 1),
                (2, 2),
                (1, 2),
                (0, 2),
                (-1, 2),
                (-2, 2),
                (-2, 1),
                (-2, 0),
                (-2, -1),
                (-2, -2),
                (-1, -2),
                (0, -2),
                (1, -2)
            ],
        );
    }
}
