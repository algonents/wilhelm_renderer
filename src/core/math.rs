/// A 4x4 column-major matrix for 2D/3D transformations.
///
/// Stored as `[f32; 16]` in column-major order, matching OpenGL's expected layout.
/// This replaces the `glam::Mat4` dependency with the subset of operations
/// actually used by this crate: identity, orthographic projection, translation,
/// rotation around Z, and scaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Column-major storage: columns[0] = first column, etc.
    cols: [f32; 16],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        cols: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    /// Create a right-handed orthographic projection with OpenGL depth range [-1, 1].
    pub fn orthographic_rh_gl(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = far - near;
        Self {
            cols: [
                2.0 / rml,           0.0,                  0.0,                0.0,
                0.0,                  2.0 / tmb,            0.0,                0.0,
                0.0,                  0.0,                 -2.0 / fmn,          0.0,
                -(right + left) / rml, -(top + bottom) / tmb, -(far + near) / fmn, 1.0,
            ],
        }
    }

    /// Create a translation matrix.
    pub fn from_translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            cols: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                x,   y,   z,   1.0,
            ],
        }
    }

    /// Create a rotation matrix around the Z axis (angle in radians).
    pub fn from_rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            cols: [
                c,   s,   0.0, 0.0,
                -s,  c,   0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a scale matrix.
    pub fn from_scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            cols: [
                x,   0.0, 0.0, 0.0,
                0.0, y,   0.0, 0.0,
                0.0, 0.0, z,   0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Return a pointer to the column-major data, for passing to OpenGL.
    #[inline]
    pub fn as_ptr(&self) -> *const f32 {
        self.cols.as_ptr()
    }
}

impl std::ops::Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let a = &self.cols;
        let b = &rhs.cols;
        let mut out = [0.0f32; 16];

        for col in 0..4 {
            for row in 0..4 {
                out[col * 4 + row] =
                    a[row]      * b[col * 4]
                  + a[4 + row]  * b[col * 4 + 1]
                  + a[8 + row]  * b[col * 4 + 2]
                  + a[12 + row] * b[col * 4 + 3];
            }
        }

        Self { cols: out }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_multiply() {
        let m = Mat4::from_translation(1.0, 2.0, 3.0);
        assert_eq!(Mat4::IDENTITY * m, m);
        assert_eq!(m * Mat4::IDENTITY, m);
    }

    #[test]
    fn orthographic_corners() {
        // For ortho(0, 800, 600, 0, -1, 1):
        // (0,0) -> (-1, 1) in NDC, (800,600) -> (1, -1) in NDC
        let p = Mat4::orthographic_rh_gl(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
        let cols = &p.cols;

        // Transform (0, 0, 0, 1): result = col3
        let x = cols[12]; // -(0+800)/800 = -1
        let y = cols[13]; // -(600+0)/600 ... let's just check the formula
        assert!((x - (-1.0)).abs() < 1e-6);
        assert!((y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn translation_multiply() {
        let t1 = Mat4::from_translation(1.0, 0.0, 0.0);
        let t2 = Mat4::from_translation(0.0, 2.0, 0.0);
        let combined = t1 * t2;
        // Translation should compose: (1,0,0) + (0,2,0) = (1,2,0)
        assert!((combined.cols[12] - 1.0).abs() < 1e-6);
        assert!((combined.cols[13] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn rotation_z_quarter_turn() {
        let r = Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2);
        // cos(90°) ≈ 0, sin(90°) ≈ 1
        assert!(r.cols[0].abs() < 1e-6);   // cos
        assert!((r.cols[1] - 1.0).abs() < 1e-6); // sin
        assert!((r.cols[4] + 1.0).abs() < 1e-6); // -sin
        assert!(r.cols[5].abs() < 1e-6);   // cos
    }

    #[test]
    fn scale_multiply() {
        let s = Mat4::from_scale(2.0, 3.0, 1.0);
        assert!((s.cols[0] - 2.0).abs() < 1e-6);
        assert!((s.cols[5] - 3.0).abs() < 1e-6);
        assert!((s.cols[10] - 1.0).abs() < 1e-6);
    }
}
