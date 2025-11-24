//! Wavefront file handling.

use crate::{
    err::Error,
    geom::SmoothTriangle,
    math::{Dir3, Point3},
};
use std::{
    io::{BufRead, BufReader},
    path::Path,
};
use obj::Obj;

pub fn mesh_from_objfile(path: &Path) -> Result<Vec<SmoothTriangle>, Error> {
    let objs = Obj::load(path)?;
    // obj.load_mtls()?

    let verts = objs.data.position.iter()
        .map(|vs| {
            let vs: Vec<f64> = vs.iter().map(|v| *v as f64).collect();
            Point3::new(vs[0], vs[1], vs[2])
        })
        .collect::<Vec<_>>();

    let norms = objs.data.normal.iter()
        .map(|vs| {
            let vs: Vec<f64> = vs.iter().map(|v| *v as f64).collect();
            Dir3::new(vs[0], vs[1], vs[2])
        })
        .collect::<Vec<_>>();

    let mut objs_faces =
        objs
        .data
        .objects
        .iter()
        .map(|obj|
            // NOTE: Don't support multiple groups per object
            obj.groups
                .iter()
                .flat_map(|group|
                    group.polys.iter().map(|poly| {
                        if poly.0.len() != 3 {
                            return Err("Only triangular faces are supported.");
                            // TODO: Could easily convert a quadrilaterl into 2 triangles, hence supporting
                            // more export options for OBJ files
                        }

                        let v_idx: Vec<_> = poly.0.iter()
                            .map(|idx_tuple| idx_tuple.0)
                            .collect();
                        //let t_idx = poly.0.map(|idx_tuple| idx_tuple.1);
                        let n_idx: Vec<_> = poly.0.iter()
                            .map(|idx_tuple| idx_tuple.2.ok_or("Missing normal index."))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(((v_idx[0], v_idx[1], v_idx[2]), (n_idx[0], n_idx[1], n_idx[2])))
                    })
                )
                .collect::<Result<Vec<_>, _>>()
        );

    let faces = objs_faces.next().ok_or("No objects found in OBJ file.")??;
    if objs_faces.next().is_some() {
        return Err(
            obj::ObjError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidData,
                    "Only one object per file is supported at the moment")
        ).into());
    }

    let mut tris = Vec::with_capacity(faces.len());
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
    use super::mesh_from_objfile;
    use crate::{
        geom::SmoothTriangle,
        math::{Dir3, Point3}};
    use std::path::Path;

    #[test]
    fn test_mesh_from_objfile() {

        // Create correct comparison data
        let tris_kgo = [SmoothTriangle::new_from_verts(
                            [Point3::new(1.0, -1.0, 0.0), Point3::new(-1.0, 1.0, 0.0), Point3::new(-1.0, -1.0, 0.0)],
                            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
                            ),
                        SmoothTriangle::new_from_verts(
                            [Point3::new(1.0, -1.0, 0.0), Point3::new(1.0, 1.0, 0.0), Point3::new(-1.0, 1.0, 0.0)],
                            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
                            )];

        let test_data_path = Path::new("./tests/data/square.obj");
        let mesh_tris = mesh_from_objfile(test_data_path).unwrap();

        assert!(mesh_tris==tris_kgo);

    }
}
