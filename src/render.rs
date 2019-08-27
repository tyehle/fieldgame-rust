use std::ops;
use graphics::Transformed;

#[derive(Copy, Clone)]
pub struct R3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl ops::Add for R3 {
    type Output = Self;

    fn add(self, other: R3) -> Self::Output {
        R3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}

impl ops::Sub for R3 {
    type Output = Self;

    fn sub(self, other: R3) -> Self::Output {
        R3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

impl ops::Mul<f64> for R3 {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        R3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other
        }
    }
}

impl R3 {
    fn normalized(&self) -> R3 {
        let norm = self.norm();
        R3 {
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm
        }
    }

    fn norm(&self) -> f64 {
        dot(&self, &self).sqrt()
    }
}


fn dot(a: &R3, b: &R3) -> f64 {
    a.x*b.x + a.y*b.y + a.z*b.z
}

fn cross(a: &R3, b: &R3) -> R3 {
    R3 {
        x: a.y*b.z - a.z*b.y,
        y: a.z*b.x - a.x*b.z,
        z: a.x*b.y - a.y*b.x
    }
}


#[derive(Copy, Clone)]
pub struct Camera {
    pub position: R3,
    pub forward: R3,
    pub right: R3,
    pub scale: f64
}

pub trait Renderable {
    fn render<G: graphics::Graphics>(&self, c: &graphics::Context, g: &mut G, camera: Camera, center: graphics::math::Matrix2d);
}

// pub struct Line {
//     a: R3,
//     b: R3
// }

// impl Renderable for Line {
//     fn render(c: &graphics::Context, g: &mut G, camera: Camera) {
//         // TODO
//     }
// }

pub struct Cube {
    pub position: R3, // smallest corner
    pub velocity: R3,
    pub size: R3,
    pub color: graphics::types::Color
}

impl Renderable for Cube {
    fn render<G: graphics::Graphics>(&self, c: &graphics::Context, g: &mut G, camera: Camera, center: graphics::math::Matrix2d) {
        let points = [
            self.position,
            self.position + R3{x: self.size.x, y: 0.0, z: 0.0},
            self.position + R3{x: 0.0, y: self.size.y, z: 0.0},
            self.position + R3{x: 0.0, y: 0.0, z: self.size.z},
            self.position + R3{x: self.size.x, y: self.size.y, z: 0.0},
            self.position + R3{x: self.size.x, y: 0.0, z: self.size.z},
            self.position + R3{x: 0.0, y: self.size.y, z: self.size.z},
            self.position + self.size,
        ];

        let point_circle = graphics::ellipse::circle(0.0, 0.0, 4.0);

        for point in points.iter() {
            let (x, y) = to_screen_space(*point, &camera);

            graphics::Ellipse::new(self.color)
                .draw(point_circle, &c.draw_state, center.trans(x, y), g);
        }
    }
}

fn to_screen_space(point: R3, camera: &Camera) -> (f64, f64) {
    let to_point = point - camera.position;
    let alpha = dot(&to_point.normalized(), &camera.forward);
    let beta = alpha / (to_point - camera.forward*dot(&to_point, &camera.forward)).norm();

    let x = beta * dot(&to_point, &camera.right);
    let y = beta * dot(&to_point, &cross(&camera.forward, &camera.right));
    (camera.scale * x, camera.scale * y)
}
