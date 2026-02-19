//! Monte-Carlo radiative transfer simulation binary.
//! Compute the radiative field for a given set of setup and light source.

use std::{
    collections::VecDeque, env::current_dir, path::{Path, PathBuf}
};
use aetherus::{
    args,
    fs::{File, Load, Save},
    geom::{Collide, Mesh, SmoothTriangle, Tree},
    ord::{Build, Link, Name, Set},
    report,
    sim::{
        Parameters, ParametersBuilderLoader
    },
    util::{
        banner::{section, sub_section, title},
        dir,
        fmt::term,
    },
};

/// Backup print width if the terminal width can not be determined.
const BACKUP_TERM_WIDTH: usize = 80;

/// Main program function.
fn main() {
    let term_width = term::width(BACKUP_TERM_WIDTH);
    title(term_width, "Aetherus Remesh");

    let (in_dir, out_dir, params_path) = initialisation(term_width);
    let params = load_parameters(term_width, &in_dir, &params_path);

    section(term_width, "Input");
    sub_section(term_width, "Reconstruction");
    let engine = params.engine;
    report!(engine, "engine");
    let sett = params.sett;
    report!(sett, "settings");
    let bound = params.boundary;
    report!(bound, "boundary");
    let mats = params.mats;
    report!(mats, "materials");

    let base_output = params.output
        .build()
        .expect("Failed to build base output.");

    sub_section(term_width, "Linking");
    let lights = params
        .lights
        .link(&mats)
        .expect("Failed to link materials to lights.");
    report!(lights, "lights");
    let attrs_future = params
        .attrs
        .link(base_output.reg.vol_reg.set())
        .expect("Failed to link volume output to attributes. ")
        .link(base_output.reg.plane_reg.set())
        .expect("Failed to link plane output to attributes. ")
        .link(base_output.reg.detectors_reg.set())
        .expect("Failed to link detectors to attributes.")
        .link(&mats)
        .expect("Failed to link materials to attributes.");
    //report!(attrs, "attributes");
    let objs_builder = params
        .objs
        .link(&attrs_future)
        .expect("Failed to link global attributes. ")
        .link(&mats)
        .expect("Failed to link materials to attributes.");

    let scenes = objs_builder
        .build()
        .expect("Failed to build scene geometries.");

    let objs: Vec<_> = scenes
        .build()
        .expect("Failed to build scene objects.")
        .into_iter()
        .flat_map(|o| o.clone())
        .collect();

    println!("{} Objects have been read from files", objs.len());
    for obj in objs.iter() {
        report!(obj.obj_name, "Object");
    }

    let surfs_vec: Vec<_> = objs
        .iter()
        .map(|obj| (Name::new(&obj.obj_name), obj.get_surface()))
        .collect();

    let mut surfs = Set::from_pairs(surfs_vec)
        .expect("Failed to build surface set.");

    //report!(surfs, "surfaces");
    let surfs_names = surfs.names_list();

    section(term_width, "Remeshing");
    // 1. Take any two objects that we try to resolve Mesh collisions for
    for i in 0..surfs_names.len() {
        for j in (i+1)..surfs_names.len() {
            if surfs_names[i] == surfs_names[j] {
                println!("[WARN] Duplicate surface name: {}", surfs_names[i]);
            }

            let surf_u = surfs.get(&surfs_names[i]).unwrap();
            let surf_v = surfs.get(&surfs_names[j]).unwrap();

            let mut surf_u_tris: VecDeque<_> = surf_u.mesh().tris().iter().map(|st| st.tri().clone()).collect();
            let mut surf_v_tris: Vec<_> = surf_v.mesh().tris().iter().map(|st| st.tri().clone()).collect();


            // Check that u_tri is free of collisions from surf_v_tris

            println!("Checking for collisions between surfaces: {} and {}", surfs_names[i], surfs_names[j]);

            let mut final_surf_u_tris = Vec::new();

            while !surf_u_tris.is_empty()
            {
                let u_tri = surf_u_tris.pop_front().unwrap();
                let mut v_tris_mutated = Vec::new();
                let mut new_surf_v_tris = Vec::new();
                let mut u_tri_collision = false;

                println!("Checking for collisions of {}:{:?} to {}", surfs_names[i], u_tri, surfs_names[j]);
                for (v_idx, v_tri) in surf_v_tris.iter().enumerate() {
                    let mut v_tri_collision = false;
                    if u_tri.overlap(v_tri) {
                        println!("Overlapping surfaces: {} and {}", surfs_names[i], surfs_names[j]);
                        println!("Overlapping between: {:?} and {:?}", u_tri, v_tri);
                        let new_u_tris = u_tri.triangle_split(&surf_v_tris[v_idx]);
                        if new_u_tris.len() > 1 {
                            println!("Splitting triangle {:?}: into {} triangles.", u_tri, new_u_tris.len());
                            new_u_tris
                                .iter()
                                .for_each(|new_u_tri|
                                    surf_u_tris.push_back(new_u_tri.clone())
                                );
                            u_tri_collision = true;
                        }
                        let new_v_tris = surf_v_tris[v_idx].triangle_split(&u_tri);
                        if new_v_tris.len() > 1 {
                            println!("Splitting triangle {:?}:{} into {} triangles.", surf_v_tris[v_idx], v_idx, new_v_tris.len());
                            surf_v_tris.remove(v_idx);
                            new_v_tris
                                .iter()
                                .for_each(|new_v_tri|
                                    surf_v_tris.push(new_v_tri.clone())
                                );
                            v_tri_collision = true;
                        }
                        assert!(new_u_tris.len() > 0 || new_v_tris.len() > 0, "Not a valid triangle collision occured");
                        break;
                    }

                    if !v_tri_collision {
                        v_tris_mutated.push(v_idx);
                        new_surf_v_tris.push(v_tri.clone());
                    }
                }

                if !u_tri_collision {
                    final_surf_u_tris.push(u_tri);
                }

                // TODO
                let mut offset = 0;
                for v_idx in v_tris_mutated {
                    surf_v_tris.remove(v_idx - offset);
                    offset += 1;
                }
                surf_v_tris.extend(new_surf_v_tris);

                *surfs.get_mut(&surfs_names[j]).unwrap().mesh_mut() =
                    Mesh::new(surf_v_tris.iter().map(|tri| SmoothTriangle::from(tri.clone())).collect());
            }
            *surfs.get_mut(&surfs_names[i]).unwrap().mesh_mut() =
                Mesh::new(final_surf_u_tris.iter().map(|tri| SmoothTriangle::from(tri.clone())).collect());
        }
    }


    surfs.save(&out_dir.join("export_surfaces.obj"))
        .expect("Failed to save surfaces to output directory.");

    sub_section(term_width, "Growing");
    let tree = Tree::new(&params.tree, &surfs);
    report!(tree, "hit-scan tree");

    section(term_width, "Saving");

    section(term_width, "Finished");
}

/// Initialise the input arguments.
fn initialisation(term_width: usize) -> (PathBuf, PathBuf, PathBuf) {
    section(term_width, "Initialisation");
    sub_section(term_width, "args");
    args!(
        bin_path:    PathBuf;
        output_dir:  PathBuf;
        input_dir:   PathBuf;
        params_path: PathBuf
    );
    report!(bin_path.display(), "binary path");
    report!(output_dir.display(), "relative output path");
    report!(input_dir.display(), "relative input path");
    report!(params_path.display(), "parameters");

    sub_section(term_width, "directories");
    let cwd = current_dir().expect("Failed to determine current working directory.");
    let (in_dir, out_dir) = dir::io_dirs(Some(cwd.join(input_dir)), Some(cwd.join(output_dir)))
        .expect("Failed to initialise directories.");
    report!(out_dir.display(), "output directory");
    report!(in_dir.display(), "input directory");

    (in_dir, out_dir, params_path)
}

/// Load the required files and form the input parameters.
fn load_parameters(term_width: usize, in_dir: &Path, params_path: &Path) -> Parameters {
    section(term_width, "Parameters");
    sub_section(term_width, "Loading");
    let builder = ParametersBuilderLoader::new_from_file(&in_dir.join(&params_path))
        .expect("Failed to load parameters file.")
        .load(&in_dir)
        .expect("Failed to load parameter resource files.");
    report!(builder, "builder");

    sub_section(term_width, "Building");
    let params = builder.build().expect("Failed to build parameters.");
    report!(params, "parameters");

    params
}
