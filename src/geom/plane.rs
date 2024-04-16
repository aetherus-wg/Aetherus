use crate::{
    geom::Ray,
    math::{Dir3, Point3}
};

pub fn ray_plane_intersection(ray: &Ray, plane_point: Point3, plane_normal: Dir3) -> Option<Point3> {
    let p0_q0 =  ray.pos() - plane_point;
    let denom = ray.dir().dot(&plane_normal);
    if denom.abs() < 1e-6 {
        // The ray and the plane are parallel -- no intersection. 
        return None;
    }
    let dist = -p0_q0.dot(&plane_normal.into()) / denom;
    if dist < 0.0 {
        // Intersection is behind the ray, in the opposite direction to the dir vector. 
        return None;
    }
    
    let ray_dir_vec = Point3::new(ray.dir().x(), ray.dir().y(), ray.dir().z());
    Some(ray_dir_vec * dist + *ray.pos())
}