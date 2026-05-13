use std::{env, path::Path, process::Command};
use aetherus::{
    err::Error, fs::{File, Load}, geom::{Surface, Tree}, io::output::Output, ord::{Build, Link, Set}, phys::{Material}, sim::{
        Attribute, Parameters, ParametersBuilderLoader
    }
};

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
    let mats = params.mats.clone();
    let base_output = params.output.clone().build().unwrap();

    // Build objects
    c.bench_function("build_objects", |b| {
        b.iter(|| {
            let (surfs, attrs) = build_objects(black_box(&params), black_box(&base_output), &mats).unwrap();
            black_box((surfs, attrs))
        });
    });

    let base_output = params.output.clone().build().unwrap();
    let (surfs, _attrs) = build_objects(&params, &base_output, &mats).unwrap();

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

    let params = builder.build().expect("Failed to build parameters.");

    params
}

fn build_objects(params: &Parameters, base_output: &Output, mats: &Set<Material>) -> Result<(Set<Surface<'static, Attribute>>, Set<Attribute>), Error> {

    let attrs = params
        .attrs
        .clone()
        .link(base_output.reg.vol_reg.set())?
        .link(base_output.reg.plane_reg.set())?
        .link(base_output.reg.phot_cols_reg.set())
        .expect("Failed to link photon collectors to attributes.")
        .link(base_output.reg.ccd_reg.set())
        .expect("Failed to link ccds to attributes.")
        .link(base_output.reg.img_reg.set())
        .expect("Failed to link imagers to attributes.")
        .link(base_output.reg.spec_reg.set())
        .expect("Failed to link spectrometers to attributes.")
        .link(&mats)
        .expect("Failed to link materials to attributes.")
        .build()
        .context("Failed to build attributes.")?;

    let attrs: &'static Set<Attribute> = Box::leak(Box::new(attrs));

    let surfs = params
        .surfs
        .clone()
        .link(&attrs)
        .expect("Failed to link attribute to surfaces.");


    Ok((surfs, attrs.clone()))
}

criterion_group!(benches, criterion_config, criterion_sim);
criterion_main!(benches);
