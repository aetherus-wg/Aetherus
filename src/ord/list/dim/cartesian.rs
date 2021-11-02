//! Cartesian dimensions.

/// Cartesian coordinate system.
pub enum Cartesian {
    /// First spatial dimension.
    X,
    /// Second spatial dimension.
    Y,
    /// Third spatial dimension.
    Z,
}

/// Cartesian X convenience indexing constant.
pub const X: usize = Cartesian::X as usize;

/// Cartesian Y convenience indexing constant.
pub const Y: usize = Cartesian::Y as usize;

/// Cartesian Z convenience indexing constant.
pub const Z: usize = Cartesian::Z as usize;

#[cfg(test)]
mod tests {
    use super::{X, Y, Z};

    /// Checking that each index pulls back the expected index. 
    #[test]
    fn cartesian_index_test() {
        let cart_vector = vec![10.0, 25.0, 32.0];
        assert_eq!(cart_vector[X], 10.0);
        assert_eq!(cart_vector[Y], 25.0);
        assert_eq!(cart_vector[Z], 32.0);
    }
}