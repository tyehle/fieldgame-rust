use std::ops;

#[derive(Copy, Clone, Debug)]
pub struct R3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl std::fmt::Display for R3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl ops::Neg for R3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        R3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Add for R3 {
    type Output = Self;

    fn add(self, other: R3) -> Self::Output {
        R3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::Sub for R3 {
    type Output = Self;

    fn sub(self, other: R3) -> Self::Output {
        R3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl ops::Mul<f64> for R3 {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        R3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl R3 {
    pub fn normalized(&self) -> R3 {
        let norm = self.norm();
        R3 {
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm,
        }
    }

    pub fn norm(&self) -> f64 {
        dot(&self, &self).sqrt()
    }
}

pub fn dot(a: &R3, b: &R3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn cross(a: &R3, b: &R3) -> R3 {
    R3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

pub fn midpoint(a: &R3, b: &R3) -> R3 {
    R3 {
        x: (a.x + b.x) * 0.5,
        y: (a.y + b.y) * 0.5,
        z: (a.z + b.z) * 0.5,
    }
}
