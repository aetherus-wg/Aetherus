//! Netcdf file handling.

use crate::{
    err::Error,
    fs::{File, Save},
    ord::cartesian::{X, Y, Z},
};
use ndarray::{Array2, Array3, ArrayView2, ArrayView3};
use netcdf::NcPutGet;
use std::path::Path;

#[allow(clippy::use_self)]
impl<T: NcPutGet> File for Array2<T> {
    #[inline]
    fn load(path: &Path) -> Result<Array2<T>, Error> {
        let file = netcdf::open(path)?;
        let data = &file.variable("data").ok_or("Missing variable 'data'.")?;
        let arr = data.values_arr::<T, _>(..).unwrap();

        let xi = arr.shape()[X];
        let yi = arr.shape()[Y];

        let arr = Array2::from_shape_vec([xi, yi], arr.into_raw_vec())?;
        Ok(arr)
    }
}

#[allow(clippy::use_self)]
impl<T: NcPutGet> File for Array3<T> {
    #[inline]
    fn load(path: &Path) -> Result<Array3<T>, Error> {
        let file = netcdf::open(path)?;
        let data = &file.variable("data").ok_or("Missing variable 'data'.")?;
        let arr = data.values_arr::<T, _>(..).unwrap();

        let xi = arr.shape()[X];
        let yi = arr.shape()[Y];
        let zi = arr.shape()[Z];

        let arr = Array3::from_shape_vec([xi, yi, zi], arr.into_raw_vec())?;
        Ok(arr)
    }
}

impl<T: NcPutGet> Save for Array2<T> {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let mut file = netcdf::create(path)?;

        let shape = self.shape();

        let dim1_name = "x";
        file.add_dimension(dim1_name, shape[X])?;
        let dim2_name = "y";
        file.add_dimension(dim2_name, shape[Y])?;

        let mut var = file.add_variable::<T>("data", &[dim1_name, dim2_name])?;
        let arr = self.as_slice().ok_or("Missing slice data.")?;
        var.put_values::<T, _>(&arr, ..).unwrap();

        Ok(())
    }
}

impl<T: NcPutGet> Save for ArrayView2<'_, T> {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let mut file = netcdf::create(path)?;

        let shape = self.shape();

        let dim1_name = "x";
        file.add_dimension(dim1_name, shape[X])?;
        let dim2_name = "y";
        file.add_dimension(dim2_name, shape[Y])?;

        let mut var = file.add_variable::<T>("data", &[dim1_name, dim2_name])?;
        let arr = self.as_slice().ok_or("Missing slice data.")?;
        var.put_values::<T, _>(&arr, ..).unwrap();

        Ok(())
    }
}

impl<T: NcPutGet> Save for Array3<T> {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let mut file = netcdf::create(path)?;

        let shape = self.shape();

        let dim1_name = "x";
        file.add_dimension(dim1_name, shape[X])?;
        let dim2_name = "y";
        file.add_dimension(dim2_name, shape[Y])?;
        let dim3_name = "z";
        file.add_dimension(dim3_name, shape[Z])?;

        let mut var = file.add_variable::<T>("data", &[dim1_name, dim2_name, dim3_name])?;
        let arr = self.as_slice().ok_or("Missing slice data.")?;
        var.put_values::<T, _>(&arr, ..).unwrap();

        Ok(())
    }
}

impl<T: NcPutGet> Save for ArrayView3<'_, T> {
    #[inline]
    fn save_data(&self, path: &Path) -> Result<(), Error> {
        let mut file = netcdf::create(path)?;

        let shape = self.shape();

        let dim1_name = "x";
        file.add_dimension(dim1_name, shape[X])?;
        let dim2_name = "y";
        file.add_dimension(dim2_name, shape[Y])?;
        let dim3_name = "z";
        file.add_dimension(dim3_name, shape[Z])?;

        let mut var = file.add_variable::<T>("data", &[dim1_name, dim2_name, dim3_name])?;
        let arr = self.as_slice().ok_or("Missing slice data.")?;
        var.put_values::<T, _>(&arr, ..).unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use ndarray::s;

    #[test]
    fn test_load_array2() {
        let arr = Array2::from_shape_vec((2, 3), vec![1, 2, 3, 4, 5, 6]).unwrap();
        let path = Path::new("test_load_array2.nc");
        arr.save_data(&path).unwrap();

        let loaded_arr = Array2::<i32>::load(&path).unwrap();
        assert_eq!(arr, loaded_arr);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_load_array3() {
        let arr = Array3::from_shape_vec((2, 3, 4), (0..2*3*4).into_iter().map(|val| val).collect::<Vec<i32>>()).unwrap();
        let path = Path::new("test_load_array3.nc");
        arr.save_data(&path).unwrap();

        let loaded_arr = Array3::<i32>::load(&path).unwrap();
        assert_eq!(arr, loaded_arr);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_save_array2() {
        let arr = Array2::from_shape_vec((2, 3), vec![1, 2, 3, 4, 5, 6]).unwrap();
        let path = Path::new("test_save_array2.nc");
        arr.save_data(&path).unwrap();

        let file = netcdf::open(&path).unwrap();
        let data = &file.variable("data").unwrap();
        let loaded_arr = data.values_arr::<i32, _>(..).unwrap().into_dimensionality().unwrap();
        assert_eq!(arr, loaded_arr);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_save_array3() {
        let arr = Array3::from_shape_vec((2, 3, 4), (0..2*3*4).into_iter().map(|val| val).collect::<Vec<i32>>()).unwrap();
        let path = Path::new("test_save_array3.nc");
        arr.save_data(&path).unwrap();

        let file = netcdf::open(&path).unwrap();
        let data = &file.variable("data").unwrap();
        let loaded_arr = data.values_arr::<i32, _>(..).unwrap().into_dimensionality().unwrap();
        assert_eq!(arr, loaded_arr);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_save_array2_view() {
        let arr = Array2::from_shape_vec((2, 3), vec![1, 2, 3, 4, 5, 6]).unwrap();
        let path = Path::new("test_save_array2_view.nc");
        arr.slice(s![.., ..]).save_data(&path).unwrap();

        let file = netcdf::open(&path).unwrap();
        let data = &file.variable("data").unwrap();
        let loaded_arr = data.values_arr::<i32, _>(..).unwrap().into_dimensionality().unwrap();
        assert_eq!(arr, loaded_arr);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_save_array3_view() {
        let arr = Array3::from_shape_vec((2, 3, 4), (0..2*3*4).into_iter().map(|val| val).collect::<Vec<i32>>()).unwrap();
        let path = Path::new("test_save_array3_view.nc");
        arr.slice(s![.., .., ..]).save_data(&path).unwrap();

        let file = netcdf::open(&path).unwrap();
        let data = &file.variable("data").unwrap();
        let loaded_arr = data.values_arr::<i32, _>(..).unwrap().into_dimensionality().unwrap();
        assert_eq!(arr, loaded_arr);

        fs::remove_file(&path).unwrap();
    }
}