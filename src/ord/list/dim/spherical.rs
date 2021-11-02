//! Spherical-polar dimensions.

/// Spherical-polar coordinate system.
pub enum Spherical {
    /// Radial distance. [0 : inf]
    Rho,
    /// Angle. [0 : Pi]
    Theta,
    /// Azimuthal angle. [0 : 2*Pi]
    Phi,
}

/// Spherical-polar and plane-polar rho convenience indexing constant.
pub const RHO: usize = Spherical::Rho as usize;

/// Spherical-polar and plane-polar theta convenience indexing constant.
pub const THETA: usize = Spherical::Theta as usize;

/// Spherical-polar phi convenience indexing constant.
pub const PHI: usize = Spherical::Phi as usize;

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use super::{RHO, THETA, PHI};

    /// Checking that each index pulls back the expected index. 
    #[test]
    fn polar_index_test() {
        let polar_vector = vec![1.0, PI, 2.0 * PI];
        assert_eq!(polar_vector[RHO], 1.0);
        assert_eq!(polar_vector[THETA], PI);
        assert_eq!(polar_vector[PHI], 2.0 * PI);
    }
}