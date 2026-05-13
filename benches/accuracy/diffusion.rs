use std::{env, f64::consts::PI, ops::Range, path::Path, process::Command};
use netcdf;
use ndarray::{self, Array3};
use integrate::prelude::*;
use plotters::prelude::*;
use anyhow::Result;

// NOTE: More generic use of custom benchmark for multiple configurations: https://bencher.dev/learn/benchmarking/rust/custom-harness/#create-a-custom-benchmark-runner

fn run_sim() {
    env::set_var("PB_QUIET", "1");

    let params_path = Path::new("scene.json5");
    let input_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion");
    let input_dir = Path::new(&input_dir_str);
    let output_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion/out");
    let output_dir = Path::new(&output_dir_str);

    let bin_path = env!("CARGO_BIN_EXE_mcrt");

    Command::new(bin_path)
        .args(&[
                output_dir.to_str().unwrap(),
                input_dir.to_str().unwrap(),
                params_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute MCRT binary.");
}

fn read_result() -> Array3<f64> {
    let output_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion/out");
    let output_dir = Path::new(&output_dir_str);
    let result_path = output_dir.join("volume_energy_density.nc");

    let file = netcdf::open(result_path)
        .expect("Failed to read result file.");

    let result = file
        .variable("data")
        .expect("Could not find variable 'data'");

    result.get::<f64, _>(..)
        .expect("Failed to read data array.")
        .into_dimensionality::<ndarray::Ix3>()
        .expect("Failed to convert data array to 3D")
}

fn pos_from_idx(idx: usize, bounds: Range<f64>, dim: usize) -> f64 {
  let bound_min = bounds.start;
  let bound_max = bounds.end;
  let delta_x = (bound_max - bound_min) / (dim as f64);
  bound_min + (idx as f64 + 0.5) * delta_x
}

fn diffusion_equation_spatial(x: f64, y: f64, z: f64,
    diff_coeff:f64,
    abs_coeff: f64
) -> f64 {
    let eff_attenuation = (abs_coeff  / diff_coeff).sqrt();
    let r = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
    (-eff_attenuation * r).exp() / (4.0 * PI * diff_coeff * r)
}

fn main() -> Result<()> {
    run_sim();

    let u_s = 500.0;
    let u_a = 10.0;
    let g = 0.4;
    //let n = 1.30;

    let u_s_reduces = u_s * (1.0 - g);
    let diff_coeff = 1.0 / (3.0 * (u_a + u_s_reduces));

    let z_idx = 10;
    let z = pos_from_idx(z_idx, 0.010..0.050, 40);

    let simulated = read_result();

    let x_min = -0.050;
    let x_max = 0.050;
    let y_min = -0.050;
    let y_max = 0.050;

    plot_phi(x_min..x_max, y_min..y_max, z, diff_coeff, u_a)?;
    plot_measure(z_idx, &simulated)?;

    let func = |x: f64, y: f64| diffusion_equation_spatial(x, y, z, diff_coeff, u_a);
    let phi_energy = simpson_rule(|x: f64|
        simpson_rule(|y: f64|
            func(x, y),
            y_min,
            y_max,
            50_usize,
        ),
        x_min,
        x_max,
        50_usize,
    ) / ((x_max - x_min) * (y_max - y_min));

    let mut measure_energy = 0.0;
    for x_idx in 0..50 {
        for y_idx in 0..50 {
            measure_energy += simulated[[x_idx, y_idx, z_idx]];
        }
    }
    measure_energy /= 50.0 * 50.0;

    println!("Phi energy: {}", phi_energy);
    println!("Measure energy: {}", measure_energy);

    let mut acc_se = 0.0;
    let mut mae = 0.0;
    for x_idx in 0..50 {
        for y_idx in 0..50 {
            let x = pos_from_idx(x_idx, -0.050..0.050, 50);
            let y = pos_from_idx(y_idx, -0.050..0.050, 50);
            let intensity_normalized = diffusion_equation_spatial(x, y, z, diff_coeff, u_a) / phi_energy;
            let measure_normalized = simulated[[x_idx, y_idx, z_idx]] / measure_energy;
            acc_se += (intensity_normalized - measure_normalized).powi(2);
            mae += (intensity_normalized - measure_normalized).abs();
        }
    }
    let rmse = (acc_se / (50.0 * 50.0)).sqrt();
    mae /= 50.0 * 50.0;

    println!("RMSE: {}", rmse);
    println!("MAE: {}", mae);

    let mut bmf = serde_json::Map::new();
    bmf.insert("accuracy_diffusion".to_string(), serde_json::json!({
        "Root Mean Squared Error": {
            "value": rmse,
        },
        "Mean Absolute Error": {
            "value": mae,
        },
    }));
    let bmf_str = serde_json::to_string_pretty(&bmf).unwrap();
    std::fs::write("results.json", &bmf_str).unwrap();
    println!("{bmf_str}");

    Ok(())
}

fn plot_measure(z_idx: usize, measured: &Array3<f64>) -> Result<()> {
    let grid = 50usize;

    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let heatmap_path = target_dir.join("diffusion_measured.png");

    let root = BitMapBackend::new(&heatmap_path, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(0..grid, 0..grid)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let mut values = Vec::with_capacity(grid * grid);
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for x_idx in 0..grid {
        for y_idx in 0..grid {
            let v = measured[[x_idx, y_idx, z_idx]];
            let v_log = (v + 1e-12).ln();
            min_val = min_val.min(v_log);
            max_val = max_val.max(v_log);
            values.push((x_idx, y_idx, v_log));
        }
    }

    chart.draw_series(values.iter().map(|(x, y, v)| {
        let t = (v - min_val) / (max_val - min_val);
        //let color = HSLColor(t, 1.0, 0.5).filled();
        let color = viridis(t);
        Rectangle::new([(*x, *y), (x + 1, y + 1)], color.filled())
    }))?;

    root.present()?;

    Ok(())
}

fn plot_phi(x_range: Range<f64>, y_range: Range<f64>, z: f64, diff_coeff: f64, abs_coeff: f64) -> Result<()> {
    let grid = 200usize;

    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let heatmap_path = target_dir.join("diffusion_expected.png");

    let root = BitMapBackend::new(&heatmap_path, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(0..grid, 0..grid)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let mut values = Vec::with_capacity(grid * grid);
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for x_idx in 0..grid {
        for y_idx in 0..grid {
            let x = pos_from_idx(x_idx, x_range.clone(), grid);
            let y = pos_from_idx(y_idx, y_range.clone(), grid);
            let v = diffusion_equation_spatial(x, y, z, diff_coeff, abs_coeff);
            let v_log = (v + 1e-12).ln();
            min_val = min_val.min(v_log);
            max_val = max_val.max(v_log);
            values.push((x_idx, y_idx, v_log));
        }
    }

    chart.draw_series(values.iter().map(|(x, y, v)| {
        let t = (v - min_val) / (max_val - min_val);
        //let color = HSLColor(t, 1.0, 0.5).filled();
        let color = viridis(t);
        Rectangle::new([(*x, *y), (x + 1, y + 1)], color.filled())
    }))?;

    root.present()?;

    Ok(())
}

// Jet colormap
#[allow(dead_code)]
fn jet(value: f64) -> RGBColor {
    let r = if value < 0.5 {
        0.0
    } else {
        255.0 * (2.0 * value - 1.0)
    };
    let g = if value < 0.5 {
        255.0 * (2.0 * value)
    } else {
        255.0 * (2.0 * (1.0 - value))
    };
    let b = if value < 0.5 {
        255.0 * (1.0 - 2.0 * value)
    } else {
        0.0
    };
    RGBColor(r as u8, g as u8, b as u8)
}

// Viridis colormap
fn viridis(value: f64) -> RGBColor {
    let t = value.clamp(0.0, 1.0);

    // Approximate Viridis control points
    let palette = [
        (68, 1, 84),    // dark purple
        (71, 44, 122),
        (59, 81, 139),
        (44, 113, 142),
        (33, 144, 141),
        (39, 173, 129),
        (92, 200, 99),
        (170, 220, 50),
        (253, 231, 37), // yellow
    ];

    let n = palette.len() - 1;
    let scaled = t * n as f64;
    let idx = scaled.floor() as usize;
    let frac = scaled - idx as f64;

    let (r1, g1, b1) = palette[idx];
    let (r2, g2, b2) = palette[(idx + 1).min(n)];

    let r = r1 as f64 + frac * (r2 as f64 - r1 as f64);
    let g = g1 as f64 + frac * (g2 as f64 - g1 as f64);
    let b = b1 as f64 + frac * (b2 as f64 - b1 as f64);

    RGBColor(r as u8, g as u8, b as u8)
}
