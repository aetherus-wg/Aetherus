# Copyright 2013-2022 Lawrence Livermore National Security, LLC and other
# Spack Project Developers. See the top-level COPYRIGHT file for details.
#
# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import sys
from spack import *

class AetherusEnv(BundlePackage):
    """A bundle package that sets the necessary environment variables needed
    to build Aetherus within a Spack environment"""

    homepage = "https://github.com/aetherus-wg/Aetherus"

    version("v0.2.0")

    depends_on("hdf5")
    depends_on("netcdf-c")

    def setup_run_environment(self, env):
        env.set("HDF5_DIR", f"{self.spec['hdf5'].prefix}")
        if sys.platform == "darwin":
            env.set("RUSTFLAGS", f"-C link-args=-L{self.spec['hdf5'].prefix}/lib, -ld_classic")
        else:
             env.set("RUSTFLAGS", f"-C link-args=-L{self.spec['hdf5'].prefix}/lib")