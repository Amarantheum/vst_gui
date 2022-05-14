use std::ops::{Mul, Index, IndexMut};
use std::ffi::CString;
use gl::types::*;
use std::fmt;

// values are in column major order
pub struct TMatrix {
    values: [f32; 9],
}

impl TMatrix {
    pub fn new(values: [[f32; 3]; 3]) -> Self {
        Self {
            values: [
                values[0][0], values[1][0], values[2][0],
                values[0][1], values[1][1], values[2][1],
                values[0][2], values[1][2], values[2][2],
            ]
        }
    }

    /// Creates a translation matrix that moves a point dx x units and dy y units.
    pub fn translation(dx: f32, dy: f32) -> Self {
        let values = [
            [1.0, 0.0, dx],
            [0.0, 1.0, dy],
            [0.0, 0.0, 1.0],
        ];
        Self::new(values)
    }

    /// Creates a rotation matrix for rotating r radians.
    pub fn rotation(r: f32) -> Self {
        let values = [
            [r.cos(), -r.sin(), 0.0],
            [r.sin(), r.cos(), 0.0],
            [0.0, 0.0, 1.0],
        ];
        Self::new(values)
    }

    /// Creates a scaling matrix for scaling by factor s
    pub fn scaling(s: f32) -> Self {
        let values = [
            [s, 0.0, 0.0],
            [0.0, s, 0.0],
            [0.0, 0.0, 1.0],
        ];
        Self::new(values)
    }

    /// Takes TRS and combines them into one matrix
    pub fn get_trs(t: Self, r: Self, s: Self) -> Self {
        t * r * s
    }

    /// transforms an input array as a point
    pub fn transform_point(&self, p: [f32; 2]) -> [f32; 2] {
        [p[0] * self[0] + p[1] * self[3] + self[6], p[0] * self[1] + p[1] * self[4] + self[7]]
    }

    /// transforms an input array as a vector
    pub fn transform_vector(&self, p: [f32; 2]) -> [f32; 2] {
        [p[0] * self[0] + p[1] * self[3], p[0] * self[1] + p[1] * self[4]]
    }

    /// returns a ptr to the beginning of the internal data buffer
    pub fn as_ptr(&self) -> *const f32 {
        self.values.as_ptr()
    }

    /// Loads the TMatrix instance as a uniform with the name "window_transform" 
    pub fn load_as_uniform(&self, program: GLuint, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            gl::UseProgram(program);
            let uniform = gl::GetUniformLocation(program, CString::new(name)?.as_ptr());
            if uniform < 0 {
                return Err(format!("GetUniformLocation(\"{}\") -> {}", name, uniform).into());
            }
            gl::UniformMatrix3fv(uniform, 1, 0, self.as_ptr());
        }
        Ok(())
    }
}

impl Mul for TMatrix {
    type Output = Self;
    fn mul(self, r: Self) -> Self {
        Self {
            values: [
                r[0] * self[0] + r[1] * self[3] + r[2] * self[6],
                r[0] * self[1] + r[1] * self[4] + r[2] * self[7],
                r[0] * self[2] + r[1] * self[5] + r[2] * self[8],
                r[3] * self[0] + r[4] * self[3] + r[5] * self[6],
                r[3] * self[1] + r[4] * self[4] + r[5] * self[7],
                r[3] * self[2] + r[4] * self[5] + r[5] * self[8],
                r[6] * self[0] + r[7] * self[3] + r[8] * self[6],
                r[6] * self[1] + r[7] * self[4] + r[8] * self[7],
                r[6] * self[2] + r[7] * self[5] + r[8] * self[8],
            ],
        }
    }
}

impl Index<usize> for TMatrix {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl IndexMut<usize> for TMatrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl Default for TMatrix {
    fn default() -> Self {
        let values = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        Self::new(values)
    }
}

impl fmt::Debug for TMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TMatrix")
         .field("values", &self.values)
         .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsafe_2d_array_ptr() {
        let x: [[f32;3];3] = [
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ];
        
        let x_ptr = &x as *const [[f32;3];3] as *const [f32; 9];
        let x_flat = unsafe { *x_ptr };
        for i in x_flat {
            println!("{}", i);
        }
    }

    #[test]
    fn test_translate() {
        let t = TMatrix::translation(1.0, 1.0);
        let v = [0.0,0.0];
        assert_eq!([1.0,1.0], t.transform_point(v));
        assert_eq!([0.0,0.0], t.transform_vector(v));

        let t = TMatrix::translation(-1.0, 1.0);
        let v = [-1.0,2.0];
        assert_eq!([-2.0,3.0], t.transform_point(v));
        assert_eq!([-1.0,2.0], t.transform_vector(v));
    }

    #[test]
    fn test_rotate() {
        let t = TMatrix::rotation(std::f32::consts::PI);
        let v = [1.0, 0.0];
        assert_eq!([-1.0,0.0], t.transform_point(v).into_iter().map(|f| f.round()).collect::<Vec<f32>>()[0..2]);
        assert_eq!([-1.0,0.0], t.transform_point(v).into_iter().map(|f| f.round()).collect::<Vec<f32>>()[0..2]);
    }

    #[test]
    fn test_scale() {
        let t = TMatrix::scaling(2.0);
        let v = [1.0, -2.0];
        assert_eq!([2.0,-4.0], t.transform_point(v));
    }

    #[test]
    fn test_trs() {
        let t = TMatrix::translation(1.0, 1.0);
        let r = TMatrix::rotation(std::f32::consts::PI / 2.0);
        let s = TMatrix::scaling(-1.0);
        let v = [1.0,1.0];
        let trs = TMatrix::get_trs(t, r, s);
        //assert_eq!([2.0,0.0], trs.transform_point(v));
        //assert_eq!([1.0,-1.0], trs.transform_vector(v));
    }

    #[test]
    fn test_debug() {
        let t = TMatrix::translation(1.0, -1.0);
        println!("{:?}", t);
    }
}