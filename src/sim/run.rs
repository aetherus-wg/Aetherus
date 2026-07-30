//! Simulation control functions.

use crate::{
    err::Error, io::output::Output, sim::{Attribute, Engine, Input}, tools::ProgressBar
};
use chacha20::ChaCha20Rng;
use rand::rand_core::{SeedableRng, Rng};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use events_ledger::prelude::*;
use events_ledger::events::Emission;

fn seed_from_master(master_seed: u64, stream_id: usize) -> [u8; 32] {
    let mut seed = [0_u8; 32];

    seed[0..8].copy_from_slice(&master_seed.to_le_bytes());
    seed[8..16].copy_from_slice(&(stream_id as u64).to_le_bytes());

    seed
}
fn seed_for_id(id: usize) -> [u8; 32] {
    let mut seed = [0_u8; 32];
    seed[..8].copy_from_slice(&(id as u64).to_le_bytes());
    seed
}


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
        .map(|id| {
            let mut rng = ChaCha20Rng::from_seed(seed_for_id(*id));
            thread(
                engine,
                input,
                &mut rng,
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
fn thread<'a, R: Rng>(
    engine: &Engine,
    input: &'a Input<'a, (Attribute, SrcId)>,
    rng: &mut R,
    mut output: Output,
    ledger_root: Arc<LedgerNode>,
    pb: &Arc<Mutex<ProgressBar>>,
) -> Output {
    let phot_energy = input.light.power() / input.sett.num_phot() as f64;

    let block_size = input.sett.block_size();
    while let Some((start, end)) = {
        let mut pb = pb.lock().expect("Could not lock progress bar.");
        let b = pb.block(block_size);
        std::mem::drop(pb);
        b
    } {
        for _ in start..end {
            let mut phot = input.light.emit(rng, phot_energy);

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
            engine.run(input, &mut output, rng, phot);
        }
    }

    output
}
