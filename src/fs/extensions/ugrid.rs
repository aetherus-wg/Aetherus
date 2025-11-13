//! Netcdf-UGRID input mesh file handling.

use crate::{
    err::Error,
    geom::{SmoothTriangle},
    math::{Dir3, Point3},
};
use netcdf;
use std::{path::Path,
          convert::TryInto};

#[allow(clippy::use_self)]

#[inline]
pub fn mesh_from_ugrid(path: &Path) -> Result<Vec<SmoothTriangle>, Error> {

    let file = netcdf::open(path)?;

    let vert_dim = &file.dimension("nMesh_node").ok_or("Missing dimension 'nMesh_node'.")?;
    let n_verts = vert_dim.len();
    let face_dim = &file.dimension("nMesh_face").ok_or("Missing dimension 'nMesh_face'.")?;
    let n_faces = face_dim.len();

    // Read mesh vertices
    let mut verts: Vec<Point3> = Vec::with_capacity(n_verts);

    let node_x = &file.variable("Mesh_node_x").ok_or("Missing variable 'Mesh_node_x'.")?;
    let px = node_x.get::<f64, _>(..).unwrap();

    let node_y = &file.variable("Mesh_node_y").ok_or("Missing variable 'Mesh_node_y'.")?;
    let py = node_y.get::<f64, _>(..).unwrap();

    let node_z = &file.variable("Mesh_node_z").ok_or("Missing variable 'Mesh_node_z'.")?;
    let pz = node_z.get::<f64, _>(..).unwrap();

    for i in 0..n_verts {
        verts.push(Point3::new(px[i], py[i], pz[i]));
    }

    // Read mesh norms
    let mut norms: Vec<Dir3> = Vec::with_capacity(n_faces);
    let normals_var = &file.variable("normal_vectors").ok_or("Missing variable 'normal_vectors'.")?;

    for n in 0..n_faces {
        norms.push(Dir3::new(normals_var.get_value::<f64, _>([n, 0])?,
                             normals_var.get_value::<f64, _>([n, 1])?,
                             normals_var.get_value::<f64, _>([n, 2])?));
    }

    // Read mesh faces
    let mut faces: Vec<((usize, usize, usize), (usize, usize, usize))> = Vec::with_capacity(n_faces);
    let faces_var = &file.variable("Mesh_face_nodes").ok_or("Missing variable 'Mesh_face_nodes'.")?;

    for n in 0..n_faces {
        faces.push((((faces_var.get_value::<u32, _>([n, 0])?-1).try_into().unwrap(),
                     (faces_var.get_value::<u32, _>([n, 1])?-1).try_into().unwrap(),
                     (faces_var.get_value::<u32, _>([n, 2])?-1).try_into().unwrap()),
                    (n, n, n)));
    }

    // Assemble mesh vector
    let mut tris = Vec::with_capacity(n_faces);
    for face in faces {
        tris.push(SmoothTriangle::new_from_verts(
            [verts[(face.0).0], verts[(face.0).1], verts[(face.0).2]],
            [norms[(face.1).0], norms[(face.1).1], norms[(face.1).2]],
        ));
    }

    Ok(tris)
}

#[cfg(test)]
mod tests {
    use super::mesh_from_ugrid;
    use crate::{
        geom::SmoothTriangle,
        math::{Dir3, Point3}};
    use std::{
        path::Path};

    #[test]
    fn test_mesh_from_ugrid() {

        // Create correct comparison data
        let tris_kgo = [SmoothTriangle::new_from_verts(
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
