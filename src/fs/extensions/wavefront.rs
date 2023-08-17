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

pub fn mesh_from_obj(path: &Path) -> Result<Vec<SmoothTriangle>, Error> {
    let vertex_lines: Vec<_> = BufReader::new(std::fs::File::open(path)?)
        .lines()
        .map(Result::unwrap)
        .filter(|line| line.starts_with("v "))
        .collect();

    let mut verts = Vec::with_capacity(vertex_lines.len());
    for line in vertex_lines {
        let mut words = line.split_whitespace();
        words.next();

        let px = words.next().ok_or("Missing vertex word.")?.parse::<f64>()?;
        let py = words.next().ok_or("Missing vertex word.")?.parse::<f64>()?;
        let pz = words.next().ok_or("Missing vertex word.")?.parse::<f64>()?;

        verts.push(Point3::new(px, py, pz));
    }

    let normal_lines: Vec<_> = BufReader::new(std::fs::File::open(path)?)
        .lines()
        .map(Result::unwrap)
        .filter(|line| line.starts_with("vn "))
        .collect();

    let mut norms = Vec::with_capacity(normal_lines.len());
    for line in normal_lines {
        let mut words = line.split_whitespace();
        words.next();

        let nx = words.next().ok_or("Missing normal word.")?.parse::<f64>()?;
        let ny = words.next().ok_or("Missing normal word.")?.parse::<f64>()?;
        let nz = words.next().ok_or("Missing normal word.")?.parse::<f64>()?;

        norms.push(Dir3::new(nx, ny, nz));
    }

    let face_lines: Vec<_> = BufReader::new(std::fs::File::open(path)?)
        .lines()
        .map(Result::unwrap)
        .filter(|line| line.starts_with("f "))
        .collect();

    let mut faces = Vec::with_capacity(face_lines.len());
    for line in face_lines {
        let line = line.replace("//", " ");
        let mut words = line.split_whitespace();
        words.next();

        let fx = words.next().ok_or("Missing face word.")?.parse::<usize>()? - 1;
        let nx = words
            .next()
            .ok_or("Missing normal word.")?
            .parse::<usize>()?
            - 1;
        let fy = words.next().ok_or("Missing face word.")?.parse::<usize>()? - 1;
        let ny = words
            .next()
            .ok_or("Missing normal word.")?
            .parse::<usize>()?
            - 1;
        let fz = words.next().ok_or("Missing face word.")?.parse::<usize>()? - 1;
        let nz = words
            .next()
            .ok_or("Missing normal word.")?
            .parse::<usize>()?
            - 1;

        faces.push(((fx, fy, fz), (nx, ny, nz)));
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
        let tris_kgo = [SmoothTriangle::new_from_verts(
                            [Point3::new(1.0, -1.0, 0.0), Point3::new(-1.0, 1.0, 0.0), Point3::new(-1.0, -1.0, 0.0)],
                            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
                            ),
                        SmoothTriangle::new_from_verts(
                            [Point3::new(1.0, -1.0, 0.0), Point3::new(1.0, 1.0, 0.0), Point3::new(-1.0, 1.0, 0.0)],
                            [Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0), Dir3::new(0.0, 0.0, 1.0)],
                            )];

        let test_data_path = Path::new("./tests/data/square.obj");
        let mesh_tris = mesh_from_obj(test_data_path).unwrap();

        assert!(mesh_tris==tris_kgo);

    }
}