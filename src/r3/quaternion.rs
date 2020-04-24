use std::ops;

use super::r3::*;

#[derive(Copy, Clone, Debug)]
pub struct Quaternion {
    pub r: f64,
    pub i: f64,
    pub j: f64,
    pub k: f64,
}

/// Multiplication is done in the same way as imaginary numbers, and then
/// reduced to a quaternion using Hamilton's rules.
impl ops::Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, other: Quaternion) -> Self::Output {
        Quaternion {
            r: self.r * other.r - self.i * other.i - self.j * other.j - self.k * other.k,
            i: self.r * other.i + self.i * other.r + self.j * other.k - self.k * other.j,
            j: self.r * other.j - self.i * other.k + self.j * other.r + self.k * other.i,
            k: self.r * other.k + self.i * other.j - self.j * other.i + self.k * other.r,
        }
    }
}

impl ops::Div<f64> for Quaternion {
    type Output = Quaternion;

    fn div(self, other: f64) -> Self::Output {
        Quaternion {
            r: self.r / other,
            i: self.i / other,
            j: self.j / other,
            k: self.k / other,
        }
    }
}

impl Quaternion {
    pub fn new(r: f64, i: f64, j: f64, k: f64) -> Quaternion {
        Quaternion { r, i, j, k }
    }

    pub fn zero_rotation() -> Quaternion {
        Quaternion::new(1.0, 0.0, 0.0, 0.0)
    }

    /// Build a quaternion from a real and imaginary parts.
    pub fn from_real_imaginary(real: f64, imaginary: &R3) -> Quaternion {
        Quaternion {
            r: real,
            i: imaginary.x,
            j: imaginary.y,
            k: imaginary.z,
        }
    }

    pub fn rotation(axis: R3, angle: f64) -> Quaternion {
        Quaternion::from_real_imaginary((angle / 2.0).cos(), &(axis * (angle / 2.0).sin()))
    }

    pub fn conjugate(&self) -> Quaternion {
        Quaternion {
            r: self.r,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }

    pub fn inverse(&self) -> Quaternion {
        let square_norm = self.r * self.r + self.i * self.i + self.j * self.j + self.k * self.k;
        self.conjugate() / square_norm
    }

    pub fn imaginary_component(&self) -> R3 {
        R3 {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }

    pub fn rotate(&self, vec: &R3) -> R3 {
        (*self * Quaternion::from_real_imaginary(0.0, vec) * self.inverse()).imaginary_component()
    }
}
