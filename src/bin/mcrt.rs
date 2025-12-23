//! Monte-Carlo radiative transfer simulation binary.
//! Compute the radiative field for a given set of setup and light source.

use std::{
    env::current_dir,
    path::{Path, PathBuf},
};
use aetherus::{
    args,
    fs::{File, Load, Save},
    geom::Tree,
    ord::{Build, Link, Name, Set},
    report,
    sim::{
        Attribute, Input, Parameters, ParametersBuilderLoader, run
    },
    util::{
        banner::{section, sub_section, title},
        dir,
        fmt::term,
    },
};
use aetherus_events::SrcId;

use std::sync::{Arc, Mutex};

/// Backup print width if the terminal width can not be determined.
const BACKUP_TERM_WIDTH: usize = 80;

/// Main program function.
fn main() {
    let term_width = term::width(BACKUP_TERM_WIDTH);
    title(term_width, "Aetherus");

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
    let mut mats = params.mats;
    report!(mats, "materials");

    let ledger = Arc::new(Mutex::new(aetherus_events::ledger::Ledger::new()));
    {
        let mut ledger_guard = ledger.lock().expect("Failed to lock ledger.");

        for name in mats.names_list() {
            let mat = mats.get_mut(&name).unwrap();
            *mat = mat.clone().with_id(ledger_guard.with_mat(name.to_string()));
        }
        drop(ledger_guard);
    }

    //sub_section(term_width, "Registration");
    //let (spec_reg, img_reg, ccd_reg, phot_col_reg) = gen_detector_registers(&params.attrs);
    // let base_output = gen_base_output(
    //     &engine,
    //     &grid,
    //     &spec_reg,
    //     &img_reg,
    //     &ccd_reg,
    //     &phot_col_reg,
    //     &params.attrs,
    // );

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


    let mut objs: Vec<_> = scenes
        .build()
        .expect("Failed to build scene objects.")
        .into_iter()
        .flat_map(|o| o.clone())
        .collect();

    for obj in objs.iter_mut() {
        let mut ledger_guard = ledger.lock().expect("Failed to lock ledger.");
        let src_id = match obj.attr {
            // TODO: Move this to allocate_ids inside Object struct
            Attribute::Interface(..) => {
                ledger_guard.with_matsurf(obj.obj_name.clone(), obj.mat_name.clone().unwrap(), None)
            },
            Attribute::Mirror(..) | Attribute::Reflector(..) => {
                ledger_guard.with_surf(obj.obj_name.clone(), None)
            },
            _ => SrcId::None,
        };
        obj.with_id(src_id).expect("Failed to assign source ID to object.");
    }

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

    let attrs = attrs_future
        .build()
        .expect("Failed to build attributes.");

    //report!(surfs, "surfaces");

    /*
     * Create a boundary for the simulation with boundary conditions.
     * For now we hard-code this to kill, but we can link this to configuration soon.
     * TODO: We probably want to implement the MPI adjacent rank transfer here too.
     */

    sub_section(term_width, "Growing");
    let tree = Tree::new(&params.tree, &surfs);
    report!(tree, "hit-scan tree");


    let nlights = lights.len();
    let data = lights
        .into_iter()
        .enumerate()
        .fold(base_output.clone(), |mut output, (light_idx, (light_id, light))| {
            section(term_width, &format!("Running for light {} ({} / {})", light_id, light_idx + 1, nlights));
            report!(light, light_id);
            let input = Input::new(&base_output.reg.spec_reg, &mats, &attrs, &light, &tree, &sett, &bound);

            let data =
                run::multi_thread(&engine, &input, &base_output, ledger.clone()).expect("Failed to run MCRT.");

            // In the case that we are outputting the files for each individual light, we can output it here with a simple setting.
            if let Some(output_individual) = sett.output_individual_lights() {
                if output_individual {
                    let indiv_outpath = out_dir.join(light_id.as_string());
                    if !indiv_outpath.exists() {
                        // Create the directory for the output if it does not already exist.
                        std::fs::create_dir(&indiv_outpath)
                            .unwrap_or_else(|_|
                                panic!("Unable to create output directory for light '{}'", light_id)
                            );
                    }
                    data.save(&indiv_outpath)
                        .unwrap_or_else(|_|
                            panic!("Failed to save output data for light '{}'", light_id)
                        );
                }
            }

            output += data;
            output
        });

    if let Some(true) = sett.uid_tracked() {
        aetherus_events::ledger::write_ledger_to_json(
            &ledger.lock().expect("Failed to lock ledger."),
            &format!("{}_ledger.json", out_dir.join("simulation").display()),
        ).expect("Failed to save ledger.");
    }

    section(term_width, "Saving");
    //report!(data, "data");
    data.save(&out_dir).expect("Failed to save output data.");

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
    let builder = ParametersBuilderLoader::new_from_file(&in_dir.join(params_path))
        .expect("Failed to load parameters file.")
        .load(in_dir)
        .expect("Failed to load parameter resource files.");
    report!(builder, "builder");

    sub_section(term_width, "Building");
    let params = builder.build().expect("Failed to build parameters.");
    report!(params, "parameters");

    params
}
