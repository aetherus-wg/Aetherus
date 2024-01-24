# Aetherus
A Monte Carlo radiative transport code, supporting complex geometry - written in Rust

[![Aetherus crate](https://img.shields.io/crates/v/Aetherus.svg)](https://crates.io/crates/Aetherus)
[![Aetherus documentation](https://docs.rs/Aetherus/badge.svg)](https://docs.rs/arctk)
![minimum rustc 1.47](https://img.shields.io/badge/rustc-1.47+-red.svg)
[![Build Status](https://travis-ci.com/aetherus-wg/Aetherus.svg?branch=main)](https://travis-ci.com/aetherus-wg/Aetherus)


## Installation
Install the rust compiler:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

System-level dependencies can be installed into a Spack environment. Spack can be installed with the following commands:

    git clone --depth=100 --branch=releases/v0.21 https://github.com/spack/spack.git ~/spack
    . ~/spack/share/spack/setup-env.sh

The Spack environment can be installed with the following commands:

    spack env activate -d .
    spack install

Check the source code:

    cargo build --all-features

Install:

    cargo install --all-features