use graphics;

use super::r3::*;
use super::render::*;

pub struct Cuboid {
    pub position: R3,
    pub velocity: R3,
    pub size: R3,
    pub color: graphics::types::Color,
}

impl Renderable for Cuboid {
    fn render(
        &self,
        c: &graphics::Context,
        g: &mut opengl_graphics::GlGraphics,
        camera: Camera,
        center: graphics::math::Matrix2d
    ) {
        // vertices
        let px = R3{x: self.position.x + self.size.x, y: self.position.y, z: self.position.z};
        let py = R3{x: self.position.x, y: self.position.y + self.size.y, z: self.position.z};
        let pz = R3{x: self.position.x, y: self.position.y, z: self.position.z + self.size.z};
        let pxy = R3{x: self.position.x + self.size.x, y: self.position.y + self.size.y, z: self.position.z};
        let pxz = R3{x: self.position.x + self.size.x, y: self.position.y, z: self.position.z + self.size.z};
        let pyz = R3{x: self.position.x, y: self.position.y + self.size.y, z: self.position.z + self.size.z};
        let pxyz = self.position + self.size;

        // render wireframe
        render_line(self.color, &self.position, &px, c, g, camera, center);
        render_line(self.color, &self.position, &py, c, g, camera, center);
        render_line(self.color, &self.position, &pz, c, g, camera, center);
        render_line(self.color, &px, &pxy, c, g, camera, center);
        render_line(self.color, &px, &pxz, c, g, camera, center);
        render_line(self.color, &py, &pxy, c, g, camera, center);
        render_line(self.color, &py, &pyz, c, g, camera, center);
        render_line(self.color, &pz, &pxz, c, g, camera, center);
        render_line(self.color, &pz, &pyz, c, g, camera, center);
        render_line(self.color, &pxy, &pxyz, c, g, camera, center);
        render_line(self.color, &pxz, &pxyz, c, g, camera, center);
        render_line(self.color, &pyz, &pxyz, c, g, camera, center);

        let mut face_color = self.color;
        face_color[3] *= 0.25;

        render_parallelogram(face_color, [self.position, pz, pxz, px], c, g, camera, center);
        render_parallelogram(face_color, [self.position, pz, pyz, py], c, g, camera, center);
        render_parallelogram(face_color, [self.position, px, pxy, py], c, g, camera, center);
        render_parallelogram(face_color, [pxyz, pxy, py, pyz], c, g, camera, center);
        render_parallelogram(face_color, [pxyz, pxy, px, pxz], c, g, camera, center);
        render_parallelogram(face_color, [pxyz, pyz, pz, pxz], c, g, camera, center);

        // let point_circle = graphics::ellipse::circle(0.0, 0.0, 4.0);

        // for point in points.iter() {
        //     let [x, y] = to_screen_space(point, &camera);

        //     graphics::Ellipse::new(self.color)
        //         .draw(point_circle, &c.draw_state, center.trans(x, y), g);
        // }
    }
}