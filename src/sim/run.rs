//! Simulation control functions.

use crate::{
    err::Error, io::output::Output, sim::{Attribute, Engine, Input}, tools::ProgressBar
};
use rand::rng;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use aetherus_events::prelude::*;
use aetherus_events::events::Emission;

/// Run a multi-threaded MCRT simulation.
/// # Errors
/// if the progress bar can not be locked.
#[allow(clippy::expect_used)]
pub fn multi_thread<'a>(
    engine: &Engine,
    input: &'a Input<'a, (Attribute, SrcId)>,
    output: &Output,
    ledger: &LedgerTree,
) -> Result<Output, Error> {
    let pb = ProgressBar::new("MCRT", input.sett.num_phot());
    let pb = Arc::new(Mutex::new(pb));

    let num_threads = input
        .sett
        .num_threads()
        .unwrap_or(usize::MAX)
        .min(num_cpus::get());
    let threads: Vec<_> = (0..num_threads).collect();
    let mut out: Vec<_> = threads
        .par_iter()
        .map(|_id| {
            thread(
                engine,
                input,
                output.clone(),
                ledger.root().clone(),
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
#[must_use]
fn thread<'a>(
    engine: &Engine,
    input: &'a Input<'a, (Attribute, SrcId)>,
    mut output: Output,
    ledger_root: Arc<LedgerNode>,
    pb: &Arc<Mutex<ProgressBar>>,
) -> Output {
    let mut rng = rng();

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
                phot = phot.with_node(
                    ledger_root
                        .insert(EventId::new_emission(Emission::GaussianBeam, SrcId::Light(0)))
                );
            }

            if input.sett.time_resolved() == Some(true) {
                phot = phot.with_time();
            }
            // FIXME: Locking here and waiting for engine to run essentially transform this into a
            // very inefficient sequential (non parallel threaded) program
            engine.run(input, &mut output, &mut rng, phot);
        }
    }

    output
}
