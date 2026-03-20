//! Monte-Carlo radiative transfer simulation binary.
//! Compute the radiative field for a given set of setup and light source.

use std::{
    env::current_dir, path::{Path, PathBuf}
};
use aetherus::{
    args,
    fs::{File, Load, Save},
    geom::Tree,
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
    env_logger::builder()
        .format_timestamp(None)
        .init();

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

    section(term_width, "Objects loading and remeshing");
    let objs: Vec<_> = scenes
        .build()
        .expect("Failed to build scene objects.")
        .into_iter()
        .flat_map(|o| o.1.clone())
        .collect();

    println!("{} Objects have been read from files", objs.len());
    for obj in objs.iter() {
        report!(obj.obj_name, "Object");
    }

    let surfs_vec: Vec<_> = objs
        .iter()
        .map(|obj| (Name::new(&obj.obj_name), obj.get_surface()))
        .collect();

    let surfs = Set::from_pairs(surfs_vec)
        .expect("Failed to build surface set.");

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
