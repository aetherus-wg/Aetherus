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

    fn at(&self, alpha: f64) -> Point3 {
        self.start + (self.end - self.start) * alpha
    }

    #[inline]
    fn intersect_unchecked(&self, other: &Segment) -> Option<(f64, f64)> {
        const EPS_INTERSECT: f64 = 1e-9;

        let u = self.end - self.start;
        let v = other.end - other.start;
        let w0 = self.start - other.start;

        let a = u.dot(&u); // u · u
        let b = u.dot(&v); // u · v
        let c = v.dot(&v); // v · v
        let d = v.dot(&w0); // v · (u1-v1)
        let e = u.dot(&w0); // u · (u1-v1)

        let det =  b * b - a * c;
        if det.abs() < EPS_INTERSECT {
            return None;
        }

        let s = (b * e - a * d) / det;
        let t = (e * c - d * b) / det;
        Some((s, t))
    }

    /// Calculate the intersection point of two line segments in 3D space.
    pub fn intersect(&self, other: &Segment) -> Option<Point3> {
        const EPS_INTERSECT: f64 = 1e-9;

        let (alpha_self, alpha_other) = self.intersect_unchecked(&other)?;

        if alpha_self < 0.0 || alpha_self > 1.0 || alpha_other < 0.0 || alpha_other > 1.0 {
            println!("Segments do not intersect within their lengths. alpha_self: {}, alpha_other: {}", alpha_self, alpha_other);
            return None;
        }

        // FIXME: This looks wrong, but it works => Revise!!!
        let u = self.end - self.start;
        let v = other.end - other.start;

        let p_self = self.start + u * alpha_other;
        let p_other = other.start + v * alpha_self;

        let p_diff = p_self - p_other;
        // Check that closest points actually coincide
        if p_diff.dot(&p_diff) <= EPS_INTERSECT * EPS_INTERSECT {
            Some(p_self)
        } else {
            println!("Closest points do not coincide. p_self: {:?}, p_other: {:?}, p_diff: {:?}", p_self, p_other, p_diff);
            None
        }
    }

    pub fn intersect_open(&self, other: &Segment) -> Option<Point3> {
        const EPS_INTERSECT: f64 = 1e-9;

        let (alpha_self, alpha_other) = self.intersect_unchecked(&other)?;

        if alpha_self <= 0.0 || alpha_self >= 1.0 || alpha_other <= 0.0 || alpha_other >= 1.0 {
            return None;
        }

        let u = self.end - self.start;
        let v = other.end - other.start;

        let p_self = self.start + u * alpha_other;
        let p_other = other.start + v * alpha_self;

        let p_diff = p_self - p_other;
        // Check that closest points actually coincide
        if p_diff.dot(&p_diff) <= EPS_INTERSECT * EPS_INTERSECT {
            Some(p_self)
        } else {
            None
        }
    }
}

