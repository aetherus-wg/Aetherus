//! Simple integration tests for MPI functionality

use mpi::{
    point_to_point::send_receive_replace_into,
    traits::*,
};
use Aetherus::{
    math::{Formula, Probability, Point3, Dir3},
    geom::{Emitter, Ray},
    phys::{Light, Material, Photon, PhotonBuf},
};
use rand;


/// Main program function
fn main() {

    // Init MPI communicator
    let comm = mpi::initialize().unwrap();
    let world = comm.world();
    let size = world.size();
    let rank = world.rank();

    if size != 2 {
        panic!("Test case only works with 2 MPI ranks");
    }

    // Init photon on home rank
    if rank == 0 {

        // Create light source
        let mut rng = rand::thread_rng();
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Dir3::new(1.0, 0.0, 0.0));
        let emitter = Emitter::new_beam(ray.clone());
        let mat = get_air_material();
        let light = Light::new(1.0, emitter, Probability::new_point(1.0), &mat);

        // Now emit a photon
        let photon = light.emit(&mut rng, 1.0);

        // Send photon across MPI rank
        let photon_buffer = PhotonBuf::new(&photon);
        world.process_at_rank(1).send(&photon_buffer);

    } else {
        let msg = world.process_at_rank(0).receive::<PhotonBuf>().0;

        let phot_return = msg.as_photon();

        assert_eq!(phot_return.ray().pos(), &Point3::new(0.0, 0.0, 0.0));
        assert_eq!(phot_return.ray().dir(), &Dir3::new(1.0, 0.0, 0.0));
        assert_eq!(phot_return.weight(), 1.0);
        assert_eq!(phot_return.wavelength(), 1.0);
        assert_eq!(phot_return.power(), 1.0);

    }

}

fn get_air_material() -> Material {
    Material::new(
        Formula::Constant { c: 1.0 }, 
        Formula::Constant { c: 1.0e-6 }, 
        None, 
        None, 
        Formula::Constant { c: 0.1 }
    )
}