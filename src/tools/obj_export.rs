use std::{collections::BTreeMap};

use obj::{Group, IndexTuple, Obj, ObjData, Object, SimplePolygon};

use crate::{err::Error, fs::Save, geom::{Mesh, Surface}, ord::Set};

fn vec_to_hash(vec: &[f32;3]) -> u128 {
    (vec[0].to_bits() as u128) |
    (vec[1].to_bits() as u128) << 32 |
    (vec[2].to_bits() as u128) << 64
}

fn mesh_to_obj(mesh: &Mesh, name: String, position_offset: usize, normal_offset: usize) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Object) {

    let mut object = Object::new(name);
    let mut group = Group::new("default".to_string());

    let positions: Vec<[f32; 3]> = mesh
        .tris()
        .iter()
        .map(|tri| tri.tri().verts())
        .flat_map(|v| v.iter().map(|v| [v.x() as f32, v.y() as f32, v.z() as f32]))
        .collect();

    let positions_map: BTreeMap<_,[f32;3]> = positions
        .iter()
        .map(|v| (vec_to_hash(v), *v))
        .collect();

    let positions_map: BTreeMap<_,([f32;3],usize)> = positions_map
        .values()
        .enumerate()
        .map(|(idx, v)| (vec_to_hash(v), (*v, idx + position_offset)))
        .collect();

    let normals: Vec<[f32; 3]> = mesh
        .tris()
        .iter()
        .map(|tri| tri.norms())
        .flat_map(|v| v.iter().map(|v| [v.x() as f32, v.y() as f32, v.z() as f32]))
        .collect();

    let normals_map: BTreeMap<_,[f32;3]> = normals
        .iter()
        .map(|v| (vec_to_hash(v), *v))
        .collect();

    let normals_map: BTreeMap<_,([f32;3],usize)> = normals_map
        .values()
        .enumerate()
        .map(|(idx, v)| (vec_to_hash(v), (*v, idx + normal_offset)))
        .collect();

    group.polys = mesh
        .tris()
        .iter()
        .map(|tri| (tri.tri().verts(), tri.norms()))
        .map(|(verts, norms)|
            SimplePolygon(
                verts.iter()
                    .zip(norms.iter())
                    .map(|(v, vn)| ([v.x() as f32, v.y() as f32, v.z() as f32], [vn.x() as f32, vn.y() as f32, vn.z() as f32]))
                    .map(|(v, vn)| IndexTuple(
                        positions_map.get(&vec_to_hash(&v)).unwrap().1, None, normals_map.get(&vec_to_hash(&vn)).map(|x| x.1)))
                    .collect()
            ))
        .collect();

    object.groups.push(group);

    let position_vec: Vec<[f32;3]> = positions_map
        .values()
        .map(|(v, idx)| (*idx, *v))
        .collect::<BTreeMap<_,_>>()
        .values()
        .cloned()
        .collect();

    let normal_vec: Vec<[f32; 3]> = normals_map
        .values()
        .map(|(v, idx)| (*idx, *v))
        .collect::<BTreeMap<_,_>>()
        .values()
        .cloned()
        .collect();


    (position_vec, normal_vec, object)
}

impl<T> Save for Set<Surface<T>> {
    fn save_data(&self, path: &std::path::Path) -> Result<(), Error> {
        let mut obj_data = ObjData::default();
        let mut position_offset = 0;
        let mut normal_offset = 0;

        for (obj_name, surf) in self.map() {
            let (position, normal, object) =
                mesh_to_obj(surf.mesh(), obj_name.as_string(), position_offset, normal_offset);
            position_offset += position.len();
            normal_offset += normal.len();
            obj_data.position.extend(position);
            obj_data.normal.extend(normal);
            obj_data.objects.push(object);
        }

        let obj = Obj { data: obj_data, path: path.to_path_buf() };

        obj.save(path)?;
        Ok(())
    }
}
