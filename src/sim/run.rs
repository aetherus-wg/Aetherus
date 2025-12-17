//! Simulation control functions.

use crate::{
    err::Error,
    sim::{Engine, Input},
    tools::ProgressBar,
    io::output::Output,
};
use rand::thread_rng;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use aetherus_events::{emission::Emission, ledger::Ledger, EventId};

/// Run a multi-threaded MCRT simulation.
/// # Errors
/// if the progress bar can not be locked.
#[allow(clippy::expect_used)]
#[inline]
pub fn multi_thread<'a>(
    engine: &Engine,
    input: Input<'a>,
    output: &Output,
    ledger: Arc<Mutex<Ledger>>,
) -> Result<Output, Error> {
    let pb = ProgressBar::new("MCRT", input.sett.num_phot());
    let pb = Arc::new(Mutex::new(pb));

    let num_threads = input
        .sett
        .num_threads()
        .unwrap_or(std::usize::MAX)
        .min(num_cpus::get());
    let threads: Vec<_> = (0..num_threads).collect();
    let mut out: Vec<_> = threads
        .par_iter()
        .map(|_id| {
            thread(
                engine,
                input.clone(),
                output.clone(),
                ledger.clone(),
                &Arc::clone(&pb),
            )
        })
        .collect();
    pb.lock()?.finish_with_message("Simulation complete.");

    let mut data = out.pop().expect("No data received.");
    while let Some(o) = out.pop() {
        data += o;
    }

    Ok(data)
}

/// Thread control function.
#[allow(clippy::expect_used)]
#[inline]
#[must_use]
fn thread<'a>(
    engine: &Engine,
    input: Input<'a>,
    mut output: Output,
    ledger: Arc<Mutex<Ledger>>,
    pb: &Arc<Mutex<ProgressBar>>,
) -> Output {
    let mut rng = thread_rng();

    let phot_energy = input.light.power() / input.sett.num_phot() as f64;

    let block_size = input.sett.block_size();
    while let Some((start, end)) = {
        let mut pb = pb.lock().expect("Could not lock progress bar.");
        let b = pb.block(block_size);
        std::mem::drop(pb);
        b
    } {
        for _ in start..end {
            let mut phot = input.light.emit(&mut rng, phot_energy);

            // FIXME: Replace emission_type and light_id placeholder witha actual values from
            // input.light
            if input.sett.uid_tracked() == Some(true) {
                *phot.uid_mut() = ledger
                    .lock()
                    .expect("Could not lock ledger.")
                    .insert_start(EventId::new_emission(Emission::GaussianBeam, 0));
            }

            if input.sett.time_resolved() == Some(true) {
                phot = phot.with_time();
            }
            // FIXME: Locking here and waiting for engine to run esentially transform this into a
            // very inefficient sequential (non parallel threaded) program
            engine.run(&input, &mut output, &ledger, &mut rng, phot);
        }
    }

    output
}
