use std::path::Path;
use aetherus::{
    err::Error, fs::{File, Load, Save}, geom::{Surface, Tree}, io::output::Output, ord::{Build, Link, Name, Set}, phys::{Light, Material}, sim::{
        Attribute, Input, Parameters, ParametersBuilderLoader, run
    }
};

use anyhow::Context;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

/// Main program function.
fn criterion_config(c: &mut Criterion) {

    let params_path = Path::new("scene.json5");
    let input_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion");
    let input_dir = Path::new(&input_dir_str);
    let output_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion/out");
    let output_dir = Path::new(&output_dir_str);

    c.bench_function("load_parameters", |b| {
        b.iter(|| {
            let params = load_parameters(black_box(&input_dir), black_box(&params_path));
            black_box(params);
        });
    });

    let params = load_parameters(&input_dir, &params_path);
    let mats = params.mats.clone();

    // Build lights
    c.bench_function("build_lights", |b| {
        b.iter(|| {
            let lights = build_lights(black_box(&params), black_box(&mats));
            black_box(lights);
        });
    });

    let lights = build_lights(&params, &mats);

    // Build Output
    c.bench_function("build_output", |b| {
        b.iter(|| {
            let base_output = black_box(&params).output.clone().build();
            assert!(base_output.is_ok());
        });
    });

    let base_output = params.output.clone().build().unwrap();

    // Build objects
    c.bench_function("build_objects", |b| {
        b.iter(|| {
            let (surfs, attrs) = build_objects(black_box(&params), black_box(&base_output), &mats).unwrap();
            black_box((surfs, attrs))
        });
    });

    let base_output = params.output.clone().build().unwrap();
    let (surfs, attrs) = build_objects(&params, &base_output, &mats).unwrap();

    c.bench_function("build_tree", |b| {
        b.iter(|| {
            let tree = Tree::new(black_box(&params.tree), black_box(&surfs));
            black_box(tree)
        });
    });

    // Build tree
    let tree = Tree::new(&params.tree, &surfs);

    // Run simulation
    let _output = run_sim(&params, attrs, &tree, &mats, lights, base_output, output_dir);

    // Save results to check that the benchmark works correctly
    //output.save(&output_dir).expect("Failed to save output data.");
}

fn criterion_sim(c: &mut Criterion) {

    let params_path = Path::new("scene.json5");
    let input_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion");
    let input_dir = Path::new(&input_dir_str);
    let output_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion/out");
    let output_dir = Path::new(&output_dir_str);

    c.bench_function("diffusion_sim", |b| {
        b.iter(|| {
            let params = load_parameters(black_box(&input_dir), black_box(&params_path));
            let mats = params.mats.clone();
            let lights = build_lights(&params, &mats);
            let base_output = black_box(&params).output.clone().build().unwrap();
            let (surfs, attrs) = build_objects(&params, &base_output, &mats).unwrap();

            let tree = Tree::new(&params.tree, &surfs);

            // Run simulation
            let output = run_sim(&params, attrs, &tree, &mats, lights, base_output, output_dir);

            black_box(output);
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

fn build_lights<'a>(params: &Parameters, mats: &'a Set<Material>) -> Set<Light<'a>> {
    params
        .lights
        .clone()
        .link(&mats)
        .expect("Failed to link materials to lights.")
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

fn run_sim(
    params: &Parameters,
    attrs: Set<Attribute>,
    tree: &Tree<Attribute>,
    mats: &Set<Material>,
    lights: Set<Light>,
    base_output: Output,
    output_dir: &Path
) -> Output {
    // Core simulation
    let engine = params.engine.clone();
    let sett = params.sett.clone();
    let bound = params.boundary.clone();
    let mut output = base_output.clone();
    for (_light_idx, (light_name, light)) in lights.into_iter().enumerate() {
        let input = Input::new(&base_output.reg.spec_reg, &mats, &attrs, light, &tree, &sett, &bound);
        let data = run::multi_thread(&engine, input, &base_output)
            .expect("Failed to run MCRT.");
        // In the case that we are outputting the files for each individual light, we can output it here with a simple setting.
        if let Some(output_individual) = sett.output_individual_lights() {
            if output_individual {
                let indiv_outpath = output_dir.join(&light_name.as_string());
                if !indiv_outpath.exists() {
                    // Create the directory for the output if it does not already exist.
                    std::fs::create_dir(&indiv_outpath).expect(&format!(
                        "Unable to create output directory for light '{}'",
                        light_name
                    ));
                }
                data.save(&indiv_outpath).expect(&format!(
                    "Failed to save output data for light '{}'",
                    light_name
                ));
            }
        }
        output += data;
    }
    output
}

criterion_group!(benches, criterion_config, criterion_sim);
criterion_main!(benches);
