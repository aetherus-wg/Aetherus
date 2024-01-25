//! Index manipulation functions.

use crate::ord::cartesian::{X, Y, Z};

/// Determine the linear index form a two-dimension index and resolution.
#[inline]
#[must_use]
pub fn two_dim_to_linear(pos: [usize; 2], res: &[usize; 2]) -> usize {
    debug_assert!(pos[X] < res[X]);
    debug_assert!(pos[Y] < res[Y]);

    (pos[Y] * res[Y]) + pos[X]
}

/// Create the next three-dimensional index from the given linear index.
#[inline]
#[must_use]
pub fn linear_to_three_dim(n: usize, res: &[usize; 3]) -> [usize; 3] {
    debug_assert!(n < (res[X] * res[Y] * res[Z]));

    let zi = n % res[Z];
    let yi = (n / res[Z]) % res[Y];
    let xi = n / (res[Y] * res[Z]);

    [xi, yi, zi]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_two_dim_to_linear() {
        let res = [2, 2];
        
        assert_eq!(two_dim_to_linear([0, 0], &res), 0);
        assert_eq!(two_dim_to_linear([1, 0], &res), 1);
        assert_eq!(two_dim_to_linear([0, 1], &res), 2);
        assert_eq!(two_dim_to_linear([1, 1], &res), 3);
    }

    #[test]
    fn test_linear_to_three_dim() {
        let res = [2, 2, 2];

        assert_eq!(linear_to_three_dim(0, &res), [0, 0, 0]);
        assert_eq!(linear_to_three_dim(1, &res), [0, 0, 1]);
        assert_eq!(linear_to_three_dim(2, &res), [0, 1, 0]);
        assert_eq!(linear_to_three_dim(3, &res), [0, 1, 1]);
        assert_eq!(linear_to_three_dim(4, &res), [1, 0, 0]);
        assert_eq!(linear_to_three_dim(5, &res), [1, 0, 1]);
        assert_eq!(linear_to_three_dim(6, &res), [1, 1, 0]);
        assert_eq!(linear_to_three_dim(7, &res), [1, 1, 1]);
    }
}