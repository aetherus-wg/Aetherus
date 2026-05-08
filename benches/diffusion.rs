use std::{
    env, path::Path, process::Command, sync::{Arc, Mutex}
};
use aetherus::{
    err::Error,
    fs::{File, Load},
    geom::{Tree, object::Object},
    io::output::Output,
    ord::{Build, Link, Name, Set},
    phys::{Light, Material},
    sim::{Attribute, Parameters, ParametersBuilderLoader}
};

use aetherus_events::prelude::*;
use anyhow::Context;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

/// Main program function.
fn criterion_config(c: &mut Criterion) {
    env::set_var("PB_QUIET", "1");

    let params_path = Path::new("scene.json5");
    let input_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion");
    let input_dir = Path::new(&input_dir_str);

    c.bench_function("load_parameters", |b| {
        b.iter(|| {
            let params = load_parameters(black_box(&input_dir), black_box(&params_path));
            black_box(params);
        });
    });

    let params = load_parameters(&input_dir, &params_path);
    let (mut ledger, mats) = build_ledger(&params);
    let _lights = build_lights(&params, &mats);
    let base_output = params.output.clone().build(()).unwrap();

    // Build objects
    c.bench_function("build_objects", |b| {
        b.iter(|| {
            let (mut ledger, mats) = build_ledger(&params);
            let (objs, attrs) = build_objects(black_box(&params), black_box(&base_output), &mut ledger, &mats).unwrap();
            black_box((objs, attrs))
        });
    });

    let (objs, _attrs) = build_objects(&params, &base_output, &mut ledger, &mats).unwrap();

    //// Build surfaces
    //c.bench_function("build_surfaces", |b| {
    //    b.iter(|| {
    //        let surfs_vec: Vec<_> = black_box(&objs)
    //            .iter()
    //            .map(|obj| (Name::new(&obj.obj_name), obj.get_surface()))
    //            .collect();
    //        let surfs = Set::from_pairs(surfs_vec).unwrap();
    //        black_box(surfs)
    //    });
    //});

    let surfs_vec: Vec<_> = objs
        .iter()
        .map(|obj| (Name::new(&obj.obj_name), obj.get_surface()))
        .collect();
    let surfs = Set::from_pairs(surfs_vec).unwrap();

    c.bench_function("build_tree", |b| {
        b.iter(|| {
            let tree = Tree::new(black_box(&params.tree), black_box(&surfs));
            black_box(tree)
        });
    });
}

fn criterion_sim(c: &mut Criterion) {
    env::set_var("PB_QUIET", "1");

    let params_path = Path::new("scene.json5");
    let input_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion");
    let input_dir = Path::new(&input_dir_str);
    let output_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion/out");
    let output_dir = Path::new(&output_dir_str);

    let bin_path = env!("CARGO_BIN_EXE_mcrt");

    c.bench_function("diffusion_sim", |b| {
        b.iter(|| {
            Command::new(bin_path)
                .args(&[
                        output_dir.to_str().unwrap(),
                        input_dir.to_str().unwrap(),
                        params_path.to_str().unwrap(),
                ])
                .output()
                .expect("Failed to execute MCRT binary.");
        });
    });
}

/// Load the required files and form the input parameters.
fn load_parameters(in_dir: &Path, params_path: &Path) -> Parameters {
    let builder = ParametersBuilderLoader::new_from_file(&in_dir.join(&params_path))
        .expect("Failed to load parameters file.")
        .load(&in_dir)
        .expect("Failed to load parameter resource files.");

    let params = builder
        .build(Name::new("simulation"))
        .expect("Failed to build parameters.");

    params
}

fn build_ledger(params: &Parameters) -> (LedgerTree, Set<Material>) {
    let mut mats = params.mats.clone();
    let mut ledger = LedgerTree::new();
    for name in mats.names_list() {
        let mat = mats.get_mut(&name).unwrap();
        *mat = mat.clone().with_id(ledger.with_mat(name.to_string()));
    }

    (ledger, mats)
}

fn build_lights<'a>(params: &Parameters, mats: &'a Set<Material>) -> Set<Light<'a>> {
    params
        .lights
        .clone()
        .link(&mats)
        .expect("Failed to link materials to lights.")
}

fn build_objects(params: &Parameters, base_output: &Output, ledger: &mut LedgerTree, mats: &Set<Material>) -> Result<(Vec<Object>, Set<Attribute>), Error> {

    let attrs_future = params
        .attrs
        .clone()
        .link(base_output.reg.vol_reg.set())?
        .link(base_output.reg.plane_reg.set())?
        .link(base_output.reg.detectors_reg.set())?
        .link(&mats)?;

    let objs_builder = params
        .objs
        .clone()
        .link(&attrs_future)?
        .link(&mats)?;

    let attrs = attrs_future
        .build(())
        .context("Failed to build attributes.")?;

    let scenes = objs_builder.build(())?;

    let mut objs: Vec<_> = scenes
        .build(())?
        .values()
        .flat_map(|o| o.clone())
        .collect();

    for obj in objs.iter_mut() {
        let src_id = match obj.attr {
            // TODO: Move this to allocate_ids inside Object struct
            Attribute::Interface(..) => {
                ledger.with_matsurf(obj.obj_name.clone(), obj.mat_name.clone().unwrap(), None)
            },
            Attribute::Mirror(..) | Attribute::Reflector(..) => {
                ledger.with_surf(obj.obj_name.clone(), None)
            },
            _ => SrcId::None,
        };
        obj.with_id(src_id).expect("Failed to assign source ID to object.");
    }

    Ok((objs, attrs))
}

criterion_group!(benches, criterion_config, criterion_sim);
criterion_main!(benches);
