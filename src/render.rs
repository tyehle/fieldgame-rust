use graphics::Transformed;
use std::ops;

#[derive(Copy, Clone, Debug)]
pub struct R3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl std::fmt::Display for R3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
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

fn midpoint(a: &R3, b: &R3) -> R3 {
    R3 {
        x: (a.x+b.x) * 0.5,
        y: (a.y+b.y) * 0.5,
        z: (a.z+b.z) * 0.5
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub position: R3,
    pub forward: R3,
    pub right: R3,
    pub scale: f64
}

pub trait Renderable {
    fn render<G: graphics::Graphics>(&self, c: &graphics::Context, g: &mut G, camera: Camera, center: graphics::math::Matrix2d);
}

pub struct Line {
    a: R3,
    b: R3,
    color: graphics::types::Color
}

impl Line {
    fn screen_points(&self, camera: Camera, resolution: f64) -> Vec<(f64, f64)> {
        let mut done = Vec::new();
        let mut todo = Vec::new();

        done.push((self.a, to_screen_space(self.a, &camera)));
        todo.push((self.b, to_screen_space(self.b, &camera)));

        const MAX_LEVEL: i32 = 5;
        let mut level = 0;
        let mut branch_done = Vec::new();
        branch_done.push(false);
        while let Some((end, (end_x, end_y))) = todo.pop() {
            let (begin, (begin_x, begin_y)) = done.last().expect("???");

            let distance = ((begin_x - end_x).powi(2) + (begin_y - end_y).powi(2)).sqrt();

            if distance <= resolution || level >= MAX_LEVEL {
                done.push((end, (end_x, end_y)));

                // complete all the branches we should
                while branch_done.pop().expect("???") {
                    level -= 1;
                }
                // note we are now going to finish this branch
                branch_done.push(true);
            } else {
                todo.push((end, (end_x, end_y)));
                let mid = midpoint(begin, &end);
                todo.push((mid, to_screen_space(mid, &camera)));
                level += 1;
                branch_done.push(false);
            }
        }

        return done.iter().map(|&x| x.1).collect();
    }
}

impl Renderable for Line {
    fn render<G: graphics::Graphics>(&self, c: &graphics::Context, g: &mut G, camera: Camera, center: graphics::math::Matrix2d) {
        let mut points = self.screen_points(camera, 50.0);
        let mut prev = points.pop().expect("???");
        while let Some(next) = points.pop() {
            graphics::Line::new(self.color, 1.0)
                .draw([prev.0, prev.1, next.0, next.1], &c.draw_state, center, g);
            // debug dots
            // if !points.is_empty() {
            //     graphics::Ellipse::new([1.0, 1.0, 1.0, 0.5])
            //         .draw(graphics::ellipse::circle(0.0, 0.0, 2.0), &c.draw_state, center.trans(next.0, next.1), g);
            // }
            prev = next;
        }
    }
}

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

        let px = R3{x: self.position.x + self.size.x, y: self.position.y, z: self.position.z};
        let py = R3{x: self.position.x, y: self.position.y + self.size.y, z: self.position.z};
        let pz = R3{x: self.position.x, y: self.position.y, z: self.position.z + self.size.z};
        let pxy = R3{x: self.position.x + self.size.x, y: self.position.y + self.size.y, z: self.position.z};
        let pxz = R3{x: self.position.x + self.size.x, y: self.position.y, z: self.position.z + self.size.z};
        let pyz = R3{x: self.position.x, y: self.position.y + self.size.y, z: self.position.z + self.size.z};
        let pxyz = self.position + self.size;

        let lines = [
            Line { a: self.position, b: px, color: self.color },
            Line { a: self.position, b: py, color: self.color },
            Line { a: self.position, b: pz, color: self.color },
            Line { a: px, b: pxy, color: self.color },
            Line { a: px, b: pxz, color: self.color },
            Line { a: py, b: pxy, color: self.color },
            Line { a: py, b: pyz, color: self.color },
            Line { a: pz, b: pxz, color: self.color },
            Line { a: pz, b: pyz, color: self.color },
            Line { a: pxy, b: pxyz, color: self.color },
            Line { a: pxz, b: pxyz, color: self.color },
            Line { a: pyz, b: pxyz, color: self.color },
        ];

        for line in lines.iter() {
            line.render(c, g, camera, center);
        }

        let point_circle = graphics::ellipse::circle(0.0, 0.0, 4.0);

        for point in points.iter() {
            let (x, y) = to_screen_space(*point, &camera);

            graphics::Ellipse::new(self.color)
                .draw(point_circle, &c.draw_state, center.trans(x, y), g);
        }
    }
}

pub fn to_screen_space(point: R3, camera: &Camera) -> (f64, f64) {
    let to_point = point - camera.position;

    let alpha = dot(&to_point.normalized(), &camera.forward).acos();

    // Don't vom when at the poles
    if alpha == 0.0 {
        (0.0, 0.0)
    } else if alpha == std::f64::consts::PI {
        (camera.scale * alpha, 0.0)
    } else {
        let beta = alpha / (to_point - camera.forward*dot(&to_point, &camera.forward)).norm();
        let x = beta * dot(&to_point, &camera.right);
        let y = beta * dot(&to_point, &cross(&camera.forward, &camera.right));
        (camera.scale * x, camera.scale * y)
    }
}
