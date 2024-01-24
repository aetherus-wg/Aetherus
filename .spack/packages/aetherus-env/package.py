import sys
from spack import *

class AetherusEnv(BundlePackage):

    version("test")

    depends_on("hdf5")
    depends_on("netcdf-c")

    def setup_run_environment(self, env):
        env.set("HDF5_DIR", f"{self.spec['hdf5'].prefix}")
        if sys.platform == "darwin":
            env.set("RUSTFLAGS", f"-C link-args=-L{self.spec['hdf5'].prefix}/lib, -ld_classic")
        else:
             env.set("RUSTFLAGS", f"-C link-args=-L{self.spec['hdf5'].prefix}/lib")