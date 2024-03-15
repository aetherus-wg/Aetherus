//! Data table implementation.

use crate::{access, err::Error, fs::Save};
use ndarray::Array2;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
    ops::AddAssign,
    path::Path,
};

/// Table of row data.
pub struct Table<T> {
    /// Data headings.
    headings: Vec<String>,
    /// Count data.
    rows: Vec<Vec<T>>,
}

impl<T> Table<T> {
    access!(rows: Vec<Vec<T>>);

    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(headings: Vec<String>, rows: Vec<Vec<T>>) -> Self {
        debug_assert!(!headings.is_empty());
        debug_assert!(!rows.is_empty());

        let num_cols = headings.len();
        for row in &rows {
            debug_assert!(row.len() == num_cols);
        }

        Self { headings, rows }
    }

    /// Deconstruct the table and yield the inner rows vector.
    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> Vec<Vec<T>> {
        self.rows
    }
}

impl<T: Copy> Table<T> {
    /// Construct a new instance from a two-dimensional array.
    #[inline]
    #[must_use]
    pub fn new_from_array(headings: Vec<String>, values: &Array2<T>) -> Self {
        debug_assert!(!headings.is_empty());
        debug_assert!(values.ncols() == headings.len());

        let num_rows = values.nrows();
        let num_cols = values.ncols();

        let mut rows = Vec::with_capacity(num_rows);
        for i in 0..num_rows {
            let mut row = Vec::with_capacity(num_cols);
            for j in 0..num_cols {
                row.push(values[[i, j]]);
            }
            rows.push(row);
        }

        Self { headings, rows }
    }
}

impl<T: AddAssign + Clone> AddAssign<&Self> for Table<T> {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        debug_assert!(self.headings == rhs.headings);
        debug_assert!(self.rows.len() == rhs.rows.len());

        for (lhs, rhs) in self.rows.iter_mut().zip(&rhs.rows) {
            for (l, r) in lhs.iter_mut().zip(rhs) {
                *l += r.clone();
            }
        }
    }
}

impl<T: Display> Save for Table<T> {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;
        write!(file, "{}", self)?;
        Ok(())
    }
}

impl<T: Display> Display for Table<T> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.headings[0])?;
        for heading in self.headings.iter().skip(1) {
            write!(fmt, ",{}", heading)?;
        }
        writeln!(fmt)?;

        for row in &self.rows {
            let mut iter = row.iter();
            if let Some(x) = iter.next() {
                write!(fmt, "{:>32}", x)?;
            }

            for x in iter {
                write!(fmt, ", {:>32}", x)?;
            }
            writeln!(fmt)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Array1;

    use super::*;

    #[test]
    fn new() {
        let headings = vec!["a".to_string(), "b".to_string()];
        let rows = vec![vec![1, 2], vec![3, 4]];
        let table = Table::new(headings, rows);
        assert_eq!(table.headings, vec!["a".to_string(), "b".to_string()]);
        // Check that we can get the rows back out, test the into_inner method in the process. 
        assert_eq!(table.into_inner(), vec![vec![1, 2], vec![3, 4]]);
    }

    /// Test the new_from_array method.
    #[test]
    fn new_from_array() {
        let headings = vec!["a".to_string(), "b".to_string()];
        let values = Array1::from_vec(vec![1, 2, 3, 4]).into_shape((2, 2)).unwrap();
        let table = Table::new_from_array(headings, &values);
        assert_eq!(table.headings, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(table.rows, vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn test_display() {
        let table = Table::new(vec!["A".to_string(), "B".to_string(), "C".to_string()], vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let table_string = format!("{}", table);
        let row_vec = table_string.strip_suffix("\n").expect("Table invalid in display test").split("\n").collect::<Vec<&str>>();

        // Check that we have three rows (1 heading + 2 data).
        assert_eq!(row_vec.len(), 3);

        // Check that the heading is correct.
        assert_eq!(row_vec[0], "A,B,C");

        // Check the formatting of the data too.
        assert_eq!(row_vec[1], "                               1,                                2,                                3");
        assert_eq!(row_vec[2], "                               4,                                5,                                6");
    }

    #[test]
    fn test_add_assign() {
        let mut table1 = Table::new(vec!["A".to_string(), "B".to_string(), "C".to_string()], vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let table2 = Table::new(vec!["A".to_string(), "B".to_string(), "C".to_string()], vec![vec![7, 8, 9], vec![10, 11, 12]]);
        table1 += &table2;
        assert_eq!(table1.rows(), &[&[8, 10, 12], &[14, 16, 18]]);
    }

    #[test]
    fn test_save_data() {
        let headings = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let rows = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let table = Table::new(headings.clone(), rows.clone());

        let path = Path::new("test.csv");
        let result = table.save_data(&path);
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, format!("A,B,C\n{:>32}, {:>32}, {:>32}\n{:>32}, {:>32}, {:>32}\n", 1, 2, 3, 4, 5, 6));

        std::fs::remove_file(&path).unwrap();
    }
}