use super::r3::*;
use super::render::*;
use super::pose::*;

pub enum FaceShape {
    Triangle([R3; 3]),
    Parallelogram([R3; 4]),
}

pub struct Face {
    pub shape: FaceShape,
    pub color: graphics::types::Color,
}

fn position(points: &FaceShape, pose: &Pose) -> FaceShape {
    let transform = |p| { pose.orientation.rotate(p) + pose.pos };

    match points {
        FaceShape::Triangle(_) => unimplemented!(),

        FaceShape::Parallelogram([a, b, c, d]) => FaceShape::Parallelogram([transform(&a), transform(&b), transform(&c) ,transform(&d)]),
    }
}

pub fn render_face(face: &Face, pose: &Pose, c: &graphics::Context, g: &mut opengl_graphics::GlGraphics, camera: Camera, center: graphics::math::Matrix2d) {
    match position(&face.shape, pose) {
        FaceShape::Triangle(vertices) => unimplemented!(), // transform(&vertices),

        FaceShape::Parallelogram(vertices) => render_parallelogram(face.color, vertices, c, g, camera, center),
    }
}