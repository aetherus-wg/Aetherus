//! Netcdf-UGRID input mesh file handling.

use crate::{
    err::Error,
    fs::{File},
    geom::{SmoothTriangle},
    math::{Dir3, Point3},
};
use ndarray::{Array1, Array3, ArrayView2, ArrayView3};
use netcdf::Numeric;
use std::path::Path;

#[allow(clippy::use_self)]

#[inline]
pub fn mesh_from_ugrid(path: &Path) -> Result<Vec<SmoothTriangle>, Error> {

    let vert_dim = &file.dimension("nMesh_node").ok_or("Missing dimension 'nMesh_node'.")?;
    let n_verts = vert_dim.len();
    let face_dim = &file.dimension("nMesh_face").ok_or("Missing dimension 'nMesh_face'.")?;
    let n_faces = face_dim.len();

    //
    let mut verts = Vec::with_capacity(n_verts);

    let node_x = &file.variable("Mesh_node_x").ok_or("Missing variable 'Mesh_node_x'.")?;
    let px = node_x.values_arr::<T, _>(..).unwrap();

    let node_y = &file.variable("Mesh_node_y").ok_or("Missing variable 'Mesh_node_y'.")?;
    let py = node_y.values_arr::<T, _>(..).unwrap();

    let node_z = &file.variable("Mesh_node_z").ok_or("Missing variable 'Mesh_node_z'.")?;
    let pz = node_z.values_arr::<T, _>(..).unwrap();

    verts.push(Point3::new(px, py, pz));

    //
    let mut norms = Vec::with_capacity(normal_lines.len());

    let normals_var = &file.variable("normal_vectors").ok_or("Missing variable 'normal_vectors'.")?;
    let normals_arr = normals_var.values_arr::<T, _>(..).unwrap();

    norms.push(Dir3::new(nx, ny, nz));

    //
    let mut tris = Vec::with_capacity(1);
}

#[cfg(test)]
mod tests {
    use super::mesh_from_obj;
    use crate::{
        geom::SmoothTriangle,
        math::{Dir3, Point3}};
    use std::{
        fs::read_dir,
        path::Path};

    #[test]
    fn test_mesh_from_obj() {

        // Create correct comparison data
        let mut tris_kgo = [SmoothTriangle::new_from_verts(
                                [Point3::new(1.0, -1.0, 0.0), Point3::new(-1.0, 1.0, 0.0), Point3::new(-1.0, -1.0, 0.0)],
                                [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
                                ),
                            SmoothTriangle::new_from_verts(
                                [Point3::new(1.0, -1.0, 0.0), Point3::new(1.0, 1.0, 0.0), Point3::new(-1.0, 1.0, 0.0)],
                                [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
                                )];

        let test_data_path = Path::new("./tests/data/square.nc");
        let mesh_tris = mesh_from_ugrid(test_data_path).unwrap();

        assert!(mesh_tris==tris_kgo);

    }
}