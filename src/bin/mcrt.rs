//! Monte-Carlo radiative transfer simulation binary.
//! Compute the radiative field for a given set of setup and light source.

use std::{
    env::current_dir,
    path::{Path, PathBuf},
};
use aetherus::{
    args,
    fs::{File, Load, Save},
    geom::{Tree, object::Object},
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
use aetherus_events::mcrt::SrcId;

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

    let base_output = params.output.build();

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
        .link(base_output.reg.phot_cols_reg.set())
        .expect("Failed to link photon collectors to attributes.")
        .link(base_output.reg.ccd_reg.set())
        .expect("Failed to link ccds to attributes.")
        .link(base_output.reg.img_reg.set())
        .expect("Failed to link imagers to attributes.")
        .link(base_output.reg.spec_reg.set())
        .expect("Failed to link spectrometers to attributes.")
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
            let input = Input::new(&base_output.reg.spec_reg, &mats, &attrs, light, &tree, &sett, &bound);

            let data =
                run::multi_thread(&engine, input, &base_output, ledger.clone()).expect("Failed to run MCRT.");

            // In the case that we are outputting the files for each individual light, we can output it here with a simple setting.
            if let Some(output_individual) = sett.output_individual_lights() {
                if output_individual {
                    let indiv_outpath = out_dir.join(&light_id.as_string());
                    if !indiv_outpath.exists() {
                        // Create the directory for the output if it does not already exist.
                        std::fs::create_dir(&indiv_outpath).expect(&format!(
                            "Unable to create output directory for light '{}'",
                            light_id
                        ));
                    }
                    data.save(&indiv_outpath).expect(&format!(
                        "Failed to save output data for light '{}'",
                        light_id
                    ));
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

// /// Generate the detector registers.
// fn gen_detector_registers(attrs: &Set<Attr>) -> (Register, Register, Register, Register) {
//     let mut spec_names = Vec::new();
//     let mut img_names = Vec::new();
//     let mut ccd_names = Vec::new();
//     let mut phot_col_names = Vec::new();

//     for attr in attrs.map().values() {
//         match *attr {
//             Attr::Spectrometer(ref name, ..) => spec_names.push(name.clone()),
//             Attr::Imager(ref name, ..) => img_names.push(name.clone()),
//             Attr::Ccd(ref name, ..) => ccd_names.push(name.clone()),
//             Attr::PhotonCollector(ref name, ..) => phot_col_names.push(name.clone()),
//             _ => {}
//         }
//     }

//     let spec_reg = Register::new(spec_names);
//     report!(spec_reg, "spectrometer register");

//     let img_reg = Register::new(img_names);
//     report!(img_reg, "imager register");

//     let ccd_reg = Register::new(ccd_names);
//     report!(ccd_reg, "ccd register");

//     let phot_col_reg = Register::new(phot_col_names);
//     report!(phot_col_reg, "photon collector register");

//     (spec_reg, img_reg, ccd_reg, phot_col_reg)
// }

// Generate the base output instance.
// fn gen_base_output<'a>(
//     engine: &Engine,
//     grid: &Grid,
//     spec_reg: &'a Register,
//     img_reg: &'a Register,
//     ccd_reg: &'a Register,
//     phot_col_reg: &'a Register,
//     attrs: &Set<Attr>,
// ) -> Output<'a> {
//     let res = *grid.res();

//     let mut specs = Vec::with_capacity(spec_reg.len());
//     for name in spec_reg.set().map().keys() {
//         for attr in attrs.values() {
//             if let Attr::Spectrometer(spec_name, [min, max], bins) = attr {
//                 if name == spec_name {
//                     specs.push(Histogram::new(*min, *max, *bins));
//                     continue;
//                 }
//             }
//         }
//     }

//     let mut imgs = Vec::with_capacity(img_reg.len());
//     let background = Colour::new(0.0, 0.0, 0.0, 1.0);
//     for name in img_reg.set().map().keys() {
//         for attr in attrs.values() {
//             if let Attr::Imager(img_name, res, _width, _center, _forward) = attr {
//                 if name == img_name {
//                     imgs.push(Image::new_blank(*res, background));
//                     continue;
//                 }
//             }
//         }
//     }

//     let mut ccds = Vec::with_capacity(ccd_reg.len());
//     for name in ccd_reg.set().map().keys() {
//         for attr in attrs.values() {
//             if let Attr::Ccd(ccd_name, res, _width, _center, _forward, binner) = attr {
//                 if name == ccd_name {
//                     ccds.push(Array3::zeros([res[X], res[Y], binner.bins() as usize]));
//                     continue;
//                 }
//             }
//         }
//     }

//     let mut photos = Vec::new();
//     if let Engine::Photo(frames, res) = engine {
//         photos.reserve(frames.len());
//         for _ in 0..frames.len() {
//             photos.push(Image::new_blank(*res, background));
//         }
//     }

//     let mut phot_cols: Vec<PhotonCollector> = Vec::new();
//     for name in phot_col_reg.set().map().keys() {
//         for attr in attrs.values() {
//             if let Attr::PhotonCollector(phot_col_id, kill_phot) = attr {
//                 if name == phot_col_id {
//                     let mut photcol = PhotonCollector::new();
//                     photcol.kill_photon = *kill_phot;
//                     phot_cols.push(photcol);
//                     continue;
//                 }
//             }
//         }
//     }

//     Output::new(
//         grid.boundary().clone(),
//         res,
//         spec_reg,
//         img_reg,
//         ccd_reg,
//         phot_col_reg,
//         specs,
//         imgs,
//         ccds,
//         photos,
//         phot_cols,
//     )
// }
