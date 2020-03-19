use gl;
use graphics::Graphics;
use graphics::Transformed;

use super::quaternion::Quaternion;
use super::r3::*;


#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub position: R3,
    pub orientation: Quaternion,
    pub scale: f64
}

pub trait Renderable {
    fn render(&self, c: &graphics::Context, g: &mut opengl_graphics::GlGraphics, camera: Camera, center: graphics::math::Matrix2d);
}

fn approximate_curve(a: &R3, b: &R3, camera: Camera, resolution: f64, max_split: i32) -> Vec<[f64; 2]> {
    let mut done = Vec::new();
    let mut todo = Vec::new();

    done.push((*a, to_screen_space(a, &camera)));
    todo.push((*b, to_screen_space(b, &camera)));

    let mut level = 0;
    let mut branch_done = Vec::new();
    branch_done.push(false);
    while let Some((end, [end_x, end_y])) = todo.pop() {
        let (begin, [begin_x, begin_y]) = done.last().expect("???");

        let distance = ((begin_x - end_x).powi(2) + (begin_y - end_y).powi(2)).sqrt();

        if distance <= resolution || level >= max_split {
            done.push((end, [end_x, end_y]));

            // complete all the branches we should
            while branch_done.pop().expect("???") {
                level -= 1;
            }
            // note we are now going to finish this branch
            branch_done.push(true);
        } else {
            todo.push((end, [end_x, end_y]));
            let mid = midpoint(begin, &end);
            todo.push((mid, to_screen_space(&mid, &camera)));
            level += 1;
            branch_done.push(false);
        }
    }

    done.iter().map(|&x| x.1).collect()
}

pub fn render_line(color: graphics::types::Color, a: &R3, b: &R3, c: &graphics::Context, g: &mut opengl_graphics::GlGraphics, camera: Camera, center: graphics::math::Matrix2d) {
    let mut points = approximate_curve(a, b, camera, 40.0, 9);
    let mut prev = points.pop().expect("???");
    while let Some(next) = points.pop() {
        graphics::Line::new(color, 1.0)
            .draw([prev[0], prev[1], next[0], next[1]], &c.draw_state, center, g);
        // debug dots
        // if !points.is_empty() {
        //     graphics::Ellipse::new([1.0, 1.0, 1.0, 0.5])
        //         .draw(graphics::ellipse::circle(0.0, 0.0, 2.0), &c.draw_state, center.trans(next[0], next[1]), g);
        // }
        prev = next;
    }
}

fn intersects_parallelogram(origin: &R3, direction: &R3, face: [R3; 4]) -> bool {
    let [a, b, _, c] = face;

    let normal = cross(&(a-b), &(a-c));
    let ao = a - *origin;
    let m = cross(direction, &ao);

    // divides are much more expensive than multiplies, so only do it once here
    let invdet = 1.0 / dot(direction, &normal);

    let t = dot(&ao, &normal) * invdet;
    let u = dot(&(a - c), &m) * invdet;
    let v = -dot(&(a - b), &m) * invdet;

    t >= 0.0 && u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0
}


fn flush_graphics(transform: graphics::math::Matrix2d, g: &mut opengl_graphics::GlGraphics) {
    let color = [0.0, 0.0, 0.0, 1.0];
    let rect = [-540.0, -540.0, 1.0, 1.0];
    graphics::Rectangle::new(color).draw(rect, &graphics::DrawState::default(), transform, g);
    g.clear_draw_state();
    graphics::Rectangle::new(color).draw(rect, &graphics::DrawState::default(), transform, g);
}


fn draw_poly(color: [f32; 4],
             poly: Vec<[f64; 2]>,
             is_behind: bool,
             draw_state: &graphics::DrawState,
             transform: graphics::math::Matrix2d,
             g: &mut opengl_graphics::GlGraphics
             ) {
    // flush any old graphics before manually messing with the draw state
    flush_graphics(transform, g);

    // cannot set blend to invert on the clip draw state
    let clip = graphics::DrawState::new_clip();
    g.use_draw_state(&clip);
    g.clear_stencil(0);
    unsafe {
        gl::StencilOp(gl::INVERT, gl::KEEP, gl::KEEP);
    }

    let p = graphics::Polygon::new([1.0, 1.0, 1.0, 1.0]);
    let anchor = poly[0];
    let mut prev = poly[1];
    for next in &poly[2..] {
        p.draw(&[anchor, prev, *next], &clip, transform, g);
        prev = *next;
    }

    if is_behind {
        // invert the stencil
        graphics::Ellipse::new([1.0, 1.0, 1.0, 1.0]).draw(graphics::rectangle::square(-540.0, -540.0, 1080.0), &clip, transform, g);
    }

    graphics::Rectangle::new(color)
        .draw([-540.0, -540.0, 1080.0, 1080.0], &graphics::DrawState::new_inside(), transform, g);

    flush_graphics(transform, g);

    // debug points
    for (n, p) in poly.iter().enumerate() {
        let shade = n as f32 / poly.len() as f32;
        graphics::Ellipse::new([1.0-shade, 1.0, shade, 1.0])
            .draw(graphics::ellipse::circle(0.0, 0.0, 2.0), draw_state, transform.trans(p[0], p[1]), g);
    }
}

pub fn render_parallelogram(
    color: [f32; 4],
    vertices: [R3; 4],
    c: &graphics::Context,
    gl: &mut opengl_graphics::GlGraphics,
    camera: Camera,
    center: graphics::math::Matrix2d
) {
    let resolution = 40.0;
    let max_split = 9;

    let mut points = Vec::new();
    let mut prev = vertices.last().expect("no vertices");
    for next in vertices.iter() {
        points.append(&mut approximate_curve(prev, next, camera, resolution, max_split));
        prev = next;
    }

    let backward = camera.orientation.rotate(&R3{x: -1.0, y: 0.0, z: 0.0});
    let is_behind = intersects_parallelogram(&camera.position, &backward, vertices);

    draw_poly(color, points, is_behind, &c.draw_state, center, gl);
}

pub fn to_screen_space(point: &R3, camera: &Camera) -> [f64; 2] {
    let to_point = *point - camera.position;

    let forward = camera.orientation.rotate(&R3{x: 1.0, y: 0.0, z: 0.0});
    let right = camera.orientation.rotate(&R3{x: 0.0, y: 1.0, z: 0.0});

    let alpha = dot(&to_point.normalized(), &forward).acos();

    // Don't vom when at the poles
    if alpha == 0.0 {
        [0.0, 0.0]
    } else if alpha == std::f64::consts::PI {
        [camera.scale * alpha, 0.0]
    } else {
        let beta = alpha / (to_point - forward*dot(&to_point, &forward)).norm();
        let x = beta * dot(&to_point, &right);
        let y = beta * dot(&to_point, &cross(&forward, &right));
        [camera.scale * x, camera.scale * y]
    }
}
