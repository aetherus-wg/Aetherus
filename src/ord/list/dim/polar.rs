//! Plane-polar dimensions.

/// Polar enumeration implementation.
pub enum Polar {
    /// Radial distance. [0 : inf]
    Rho,
    /// Angle. [0 : 2*Pi]
    Theta,
}

/// Polar Rho convenience indexing constant. 
pub const RHO: usize = Polar::Rho as usize;

/// Polar Rho convenience indexing constant. 
pub const THETA: usize = Polar::Theta as usize;

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use super::{RHO, THETA};

    /// Checking that each index pulls back the expected index. 
    #[test]
    fn polar_index_test() {
        let polar_vector = vec![1.0, 2.0 * PI];
        assert_eq!(polar_vector[RHO], 1.0);
        assert_eq!(polar_vector[THETA], 2.0 * PI);
    }
}