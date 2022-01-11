//! Comma-Separated-Variable file handling.

use crate::{data::Table, err::Error, fs::File};
use std::{
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

impl<T: FromStr> File for Table<T> {
    #[inline]
    fn load(path: &Path) -> Result<Self, Error> {
        // Load all of the lines into a vector of lines.
        let mut lines: Vec<_> = BufReader::new(std::fs::File::open(path)?)
            .lines()
            .map(Result::unwrap)
            .filter(|line| !line.starts_with("//"))
            .collect();

        // As we know the number of rows, we can pre-allocate the rows vector.
        let mut rows = Vec::with_capacity(lines.len());
        // We make the reasonable assumption that the CSV file has a header on the first row.
        let headings = lines
            .remove(0)
            .split(',')
            .map(|s| (*s).to_string())
            .collect();
        // Now iterate the remaining lines, attempt to parse them and push them onto the rows vec.
        for mut line in lines {
            line.retain(|c| !c.is_whitespace());
            let row = line
                .split(',')
                .map(str::parse)
                .filter_map(Result::ok)
                .collect();
            rows.push(row);
        }

        Ok(Self::new(headings, rows))
    }
}
