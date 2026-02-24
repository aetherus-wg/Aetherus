use core::f64;

use log::trace;

use crate::math::Point3;

#[derive(Debug)]
pub struct Segment {
    pub start: Point3,
    pub end: Point3,
}

impl Segment {
    pub fn new(start: Point3, end: Point3) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> f64 {
        nalgebra::distance(&self.start.data(), &self.end.data())
    }

    #[inline]
    fn at(&self, alpha: f64) -> Point3 {
        self.start + (self.end - self.start) * alpha
    }

    #[inline]
    fn intersect_unchecked(&self, other: &Segment) -> Option<(f64, f64)> {
        const EPS_INTERSECT: f64 = 1e-9;

        // Check if segments are colinear
        let u = self.end - self.start;
        let v = other.end - other.start;
        if u.cross(&v).abs() < f64::EPSILON {
            return None;
        }

        let u = self.end - self.start;
        let v = other.end - other.start;
        let w0 = self.start - other.start;

        let a = u.dot(&u); // u · u
        let b = u.dot(&v); // u · v
        let c = v.dot(&v); // v · v
        let d = u.dot(&w0); // u · (u1-v1)
        let e = v.dot(&w0); // v · (u1-v1)

        let det =  a * c - b * b;
        if det.abs() < EPS_INTERSECT {
            return None;
        }

        let alpha_u = (b * e - c * d) / det;
        let alpha_v = (a * e - b * d) / det;
        //let s = (b * e - a * d) / det;
        //let t = (e * c - d * b) / det;
        Some((alpha_u, alpha_v))
    }

    /// Calculate the intersection point of two line segments in 3D space.
    pub fn intersect(&self, other: &Segment) -> Option<Point3> {
        const EPS_INTERSECT: f64 = 1e-9;

        // Check if segments are colinear
        let u = self.end - self.start;
        let v = other.end - other.start;
        if u.cross(&v).abs() < f64::EPSILON {
            return None;
        }

        let (alpha_u, alpha_v) = self.intersect_unchecked(&other)?;

        if alpha_u < 0.0 || alpha_u > 1.0 || alpha_v < 0.0 || alpha_v > 1.0 {
            trace!("Segments do not intersect within their lengths. alpha_self: {}, alpha_other: {}", alpha_u, alpha_v);
            return None;
        }

        let p_u = self.at(alpha_u);
        let p_v = other.at(alpha_v);

        let p_diff = p_u - p_v;
        // Check that closest points actually coincide
        if p_diff.dot(&p_diff) <= EPS_INTERSECT * EPS_INTERSECT {
            Some(p_u)
        } else {
            trace!("Closest points do not coincide. p_self: {:?}, p_other: {:?}, p_diff: {:?}", p_u, p_v, p_diff);
            None
        }
    }

    /// Calculate the intersection point of two line segments in 3D space.
    pub fn intersect_with_eps(&self, other: &Segment, eps: f64) -> Option<(Point3, bool)> {
        // Check if segments are colinear
        let u = self.end - self.start;
        let v = other.end - other.start;
        if u.cross(&v).abs() < f64::EPSILON {
            return None;
        }

        let (alpha_u, alpha_v) = self.intersect_unchecked(&other)?;

        if alpha_u < 0.0 || alpha_u > 1.0 || alpha_v < -eps || alpha_v > 1.0 + eps {
            trace!("Segments do not intersect within their lengths. alpha_self: {}, alpha_other: {}", alpha_u, alpha_v);
            return None;
        }

        let fuzzy = alpha_v < eps || alpha_v > 1.0 - eps;

        let p_u = self.at(alpha_u);
        let p_v = other.at(alpha_v);

        let p_diff = p_u - p_v;
        // Check that closest points actually coincide
        if p_diff.dot(&p_diff) <= eps*eps {
            Some((p_u, fuzzy))
        } else {
            trace!("Closest points do not coincide. p_self: {:?}, p_other: {:?}, p_diff: {:?}", p_u, p_v, p_diff);
            None
        }
    }

    pub fn intersect_open(&self, other: &Segment) -> Option<Point3> {
        const EPS_INTERSECT: f64 = 1e-9;

        let (alpha_u, alpha_v) = self.intersect_unchecked(&other)?;

        if alpha_u < 0.0 || alpha_u > 1.0 || alpha_v <= 0.0 || alpha_v >= 1.0 {
            return None;
        }

        let p_u = self.at(alpha_u);
        let p_v = other.at(alpha_v);

        let p_diff = p_u - p_v;
        // Check that closest points actually coincide
        if p_diff.dot(&p_diff) <= EPS_INTERSECT * EPS_INTERSECT {
            Some(p_u)
        } else {
            None
        }
    }

    pub fn colinear(&self, other: &Segment) -> bool {
        let u = self.end - self.start;
        let v = other.end - other.start;
        let delta = other.end - self.start;
        u.cross(&v).abs() <= f64::EPSILON && u.cross(&delta).abs() <= f64::EPSILON
    }

    pub fn colinear_with_eps(&self, other: &Segment, eps: f64) -> bool {
        let u = self.end - self.start;
        let v = other.end - other.start;
        u.cross(&v).abs() <= eps
    }
}


#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;

    use super::*;

    #[test]
    fn test_intersection() {
        let seg1 = Segment::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
        let seg2 = Segment::new(Point3::new(0.0, -1.0, 0.0), Point3::new(0.0, 1.0, 0.0));
        assert!(matches!(seg1.intersect(&seg2), Some(_)));
        assert!(matches!(seg2.intersect_open(&seg1), None));

        let seg2 = Segment::new(Point3::new(0.0, -1.0, 0.0), Point3::new(0.0, 0.0, 0.0));
        assert!(matches!(seg1.intersect(&seg2), Some(_)));
        assert!(matches!(seg1.intersect_open(&seg2), None));

        let seg2 = Segment::new(Point3::new(1e-9, -1.0, 0.0), Point3::new(1e-9, 1.0, 0.0));
        assert!(matches!(seg1.intersect(&seg2), Some(_)));
        assert!(matches!(seg2.intersect_open(&seg1), Some(_)));

    }

    #[test]
    fn test_colinear() {
        let seg1 = Segment::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let seg2 = Segment::new(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 2.0, 2.0));
        assert!(seg1.colinear(&seg2));
        assert!(seg1.colinear_with_eps(&seg2, 1e-9));
    }

    #[test]
    fn test_intersection_unchecked() {
        let seg1 = Segment::new(Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 1.0));
        let seg2 = Segment::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 2.0, 2.0));

        let (alpha_u, alpha_v) = seg1.intersect_unchecked(&seg2).unwrap();
        assert_approx_eq!(alpha_u, 0.5);
        assert_approx_eq!(alpha_v, 0.25);
        assert!(matches!(seg1.intersect(&seg2), Some(_)));
        assert!(matches!(seg1.intersect_open(&seg2), Some(_)));
        assert!(matches!(seg1.intersect_with_eps(&seg2, 1e-9), Some(_)));

    }

}
