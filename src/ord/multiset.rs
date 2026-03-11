use std::{
    fmt::Display,
    path::Path
};
use crate::{
    fmt_report,
    ord::Set,
    err::Error,
    fs::{Redirect, Load},
};
use serde::{Serialize, Deserialize};
use anyhow::Context;

/// Is intended to allow us to load an array of Redirects from different files, both
/// inline (Here), and in different files (There). With this, multiple files can
/// be included for any of the parameters for which this is used.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MultiSet<T> {
    Single(Redirect<Set<T>>),
    Multi(Vec<Redirect<Set<T>>>),
}

impl<T> MultiSet<T>
where
    for<'de> T: Deserialize<'de>,
{
    pub fn load(self, in_dir: &Path) -> Result<Set<T>, Error> {
        match self {
            Self::Single(redir) => Load::load(redir, in_dir),
            Self::Multi(redirs) => {
                redirs.into_iter()
                    .fold(Ok(Set::empty()), |acc, redir| {
                        let curr_set = Load::load(redir, in_dir).context("Set load from MultiSet")?;
                        acc?.combine(curr_set)
                    })
            }
        }
    }
}

impl<T: Display> Display for MultiSet<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Single(ref set) => {
                writeln!(fmt, "{}", set)?;
            },
            Self::Multi(ref ms) => {
                writeln!(fmt, "MultiSet: ")?;
                fmt_report!(fmt, ms.iter().count(), "length");
            }
        };
        Ok(())
    }
}
