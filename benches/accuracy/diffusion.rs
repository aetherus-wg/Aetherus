use std::{env, f64::consts::PI, io::{self, Write}, ops::Range, path::Path, process::Command};
use netcdf;
use ndarray::{self, Array2, Array3};
use integrate::prelude::*;
use plotters::prelude::*;
use anyhow::Result;

// NOTE: More generic use of custom benchmark for multiple configurations: https://bencher.dev/learn/benchmarking/rust/custom-harness/#create-a-custom-benchmark-runner

fn run_sim() {
    env::set_var("PB_QUIET", "1");

    let params_path = Path::new("scene_accuracy.json5");
    let input_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion");
    let input_dir = Path::new(&input_dir_str);
    let output_dir_str = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "benches/data/diffusion/out");
    let output_dir = Path::new(&output_dir_str);

    let bin_path = env!("CARGO_BIN_EXE_mcrt");
    let mut command = Command::new(bin_path);
    command.args(&[
                output_dir.to_str().unwrap(),
                input_dir.to_str().unwrap(),
                params_path.to_str().unwrap(),
        ]);

    println!("Running MCRT binary with command: {:?}", command);

    let output = command.output().expect("Failed to execute MCRT binary.");
    io::stderr().write_all(&output.stderr).unwrap();
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

    let u_s = 50.0;
    let u_a = 1.0;
    let g = 0.4;
    //let n = 1.30;
    let x_min = -0.050;
    let x_max = 0.050;
    let y_min = -0.050;
    let y_max = 0.050;
    let x_steps = 50;
    let y_steps = 50;
    let z_steps = 40;
    let xy_cells = x_steps * y_steps;

    let u_s_reduces = u_s * (1.0 - g);
    let diff_coeff = 1.0 / (3.0 * (u_a + u_s_reduces));

    let z_idx = 10;
    let z = pos_from_idx(z_idx, 0.010..0.050, z_steps);

    let simulated = read_result();

    plot_phi(x_min..x_max, y_min..y_max, (x_steps, y_steps), z, diff_coeff, u_a)?;
    plot_measure(z_idx, &simulated)?;

    let func = |x: f64, y: f64| diffusion_equation_spatial(x, y, z, diff_coeff, u_a);
    let phi_sampled = sample(func, x_min..x_max, y_min..y_max, (x_steps, y_steps));
    plot_sampled((x_steps, y_steps), &phi_sampled)?;

    let mut simulated_average = 0.0;
    for x_idx in 0..x_steps {
        for y_idx in 0..y_steps {
            simulated_average += simulated[[x_idx, y_idx, z_idx]];
        }
    }
    simulated_average /= xy_cells as f64;

    let mut phi_average = 0.0;
    for x_idx in 0..x_steps {
        for y_idx in 0..y_steps {
            phi_average += phi_sampled[[x_idx, y_idx]];
        }
    }
    phi_average /= xy_cells as f64;

    println!("Phi average: {}", phi_average);
    println!("Simulated average: {}", simulated_average);

    let mut rel_acc_se = 0.0;
    let mut acc_se = 0.0;
    let mut mae = 0.0;
    for x_idx in 0..x_steps {
        for y_idx in 0..y_steps {
            let simulated_normalized = simulated[[x_idx, y_idx, z_idx]] / simulated_average;
            let phi_normalized = phi_sampled[[x_idx, y_idx]] / phi_average;
            let err = phi_normalized - simulated_normalized;
            rel_acc_se += (err / phi_normalized).powi(2);
            acc_se += (err).powi(2);
            mae += (err).abs();
        }
    }
    let rmse = (acc_se / xy_cells as f64).sqrt();
    let rmsre = (rel_acc_se / xy_cells as f64).sqrt();
    mae /= xy_cells as f64;

    let tv_l1 = total_variation_l1(&simulated) / (50.0 * 50.0) / simulated_average;

    println!("RMSE: {}", rmse);
    println!("RMSRE: {}", rmsre);
    println!("MAE: {}", mae);
    println!("Total Variation (L1): {}", tv_l1);

    let mut bmf = serde_json::Map::new();
    bmf.insert("accuracy_diffusion".to_string(), serde_json::json!({
        "Root Mean Squared Error": {
            "value": rmse,
        },
        "Root Mean Squared Relative Error": {
            "value": rmsre,
        },
        "Mean Absolute Error": {
            "value": mae,
        },
        "Total Variation L1": {
            "value": tv_l1,
        },
    }));
    let bmf_str = serde_json::to_string_pretty(&bmf).unwrap();
    std::fs::write("results.json", &bmf_str).unwrap();
    println!("{bmf_str}");

    Ok(())
}

fn sample(func: impl Fn(f64, f64) -> f64, x_range: Range<f64>, y_range: Range<f64>, (x_steps, y_steps): (usize, usize)) -> Array2<f64> {
    let mut result = Array2::zeros((x_steps, y_steps));
    let dx = (x_range.end - x_range.start) / (x_steps as f64);
    let dy = (y_range.end - y_range.start) / (y_steps as f64);
    for x_idx in 0..x_steps {
        for y_idx in 0..y_steps {
            let x = x_range.start + x_idx as f64 * dx;
            let y = y_range.start + y_idx as f64 * dy;
            let dphi_int = simpson_rule(|u: f64|
                simpson_rule(|v: f64|
                    func(u, v),
                    y,
                    y + dy,
                    32_usize,
                ),
                x,
                x + dx,
                32_usize,
            );
            result[[x_idx, y_idx]] = dphi_int / (dx * dy);
        }
    }
    result
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
            min_val = min_val.min(v);
            max_val = max_val.max(v);
            values.push((x_idx, y_idx, v));
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

fn plot_sampled((x_steps, y_steps): (usize, usize), value: &Array2<f64>) -> Result<()> {
    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let heatmap_path = target_dir.join("diffusion_sampled.png");

    let root = BitMapBackend::new(&heatmap_path, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(0..x_steps, 0..y_steps)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let mut values = Vec::with_capacity(x_steps * y_steps);
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for x_idx in 0..x_steps {
        for y_idx in 0..y_steps {
            let v = value[[x_idx, y_idx]];
            min_val = min_val.min(v);
            max_val = max_val.max(v);
            values.push((x_idx, y_idx, v));
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

fn plot_phi(x_range: Range<f64>, y_range: Range<f64>, (x_steps, y_steps): (usize, usize), z: f64, diff_coeff: f64, abs_coeff: f64) -> Result<()> {
    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let heatmap_path = target_dir.join("diffusion_expected.png");

    let root = BitMapBackend::new(&heatmap_path, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(0..x_steps, 0..y_steps)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let mut values = Vec::with_capacity(x_steps * y_steps);
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for x_idx in 0..x_steps {
        for y_idx in 0..y_steps {
            let x = pos_from_idx(x_idx, x_range.clone(), x_steps);
            let y = pos_from_idx(y_idx, y_range.clone(), y_steps);
            let v = diffusion_equation_spatial(x, y, z, diff_coeff, abs_coeff);
            min_val = min_val.min(v);
            max_val = max_val.max(v);
            values.push((x_idx, y_idx, v));
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

/// Total Variation using L1 norm
fn total_variation_l1(image: &Array3<f64>) -> f64 {
    let mut tv = 0.0;
    let (x_dim, y_dim, z_dim) = image.dim();
    for x in 0..x_dim {
        for y in 0..y_dim {
            for z in 0..z_dim {
                if x < x_dim - 1 {
                    tv += (image[[x + 1, y, z]] - image[[x, y, z]]).abs();
                }
                if y < y_dim - 1 {
                    tv += (image[[x, y + 1, z]] - image[[x, y, z]]).abs();
                }
                if z < z_dim - 1 {
                    tv += (image[[x, y, z + 1]] - image[[x, y, z]]).abs();
                }
            }
        }
    }
    tv
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
