use std::convert::TryInto;

use gl;
use graphics::Graphics;
use graphics::Transformed;

use super::quaternion::Quaternion;
use super::r3::*;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub position: R3,
    pub orientation: Quaternion,
    pub scale: f64,
}

pub trait Renderable {
    fn render(
        &self,
        c: &graphics::Context,
        g: &mut opengl_graphics::GlGraphics,
        camera: Camera,
        center: graphics::math::Matrix2d,
    );
}

/// The difference between two angles
/// Inputs should be between -pi and pi, and the output will between -pi and pi.
pub fn angle_difference(start: f64, end: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let angle = end - start;
    if angle > pi {
        angle - 2.0*pi
    } else if angle < -pi {
        angle + 2.0*pi
    } else {
        angle
    }
}

/// Checks if a point is behind the camera
fn is_behind(p: &R3, camera: &Camera) -> bool {
    let forward = camera.orientation.rotate(&R3::new(1.0, 0.0, 0.0));
    return dot(&(*p - camera.position), &forward) < 0.0;
}

/// Push a set of points approximating a circle arc between start and end
fn approximate_circle<F>(
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    mut push_result: F
) where F: FnMut([f64; 2]) {
    const CIRCLE_RES: f64 = 0.1; // min point spacing in radians

    let start_radius = (start_x.powi(2) + start_y.powi(2)).sqrt();
    let end_radius = (end_x.powi(2) + end_y.powi(2)).sqrt();

    let start_angle = start_y.atan2(start_x);
    let end_angle = end_y.atan2(end_x);

    // find angle between start and end
    let angle_span = angle_difference(start_angle, end_angle);
    let count = (angle_span.abs() / CIRCLE_RES).ceil() as i32;
    let step = angle_span / (count as f64);
    let radius_step = (end_radius - start_radius) / (count as f64);

    // add each point
    let mut i = 1;
    let mut a = start_angle + step;
    let mut r = start_radius + radius_step;
    loop {
        if i >= count {
            break;
        }
        push_result([a.cos() * r, a.sin() * r]);
        i += 1;
        a += step;
        r += radius_step;
    }

    // println!("angle_span: {:.2}, count: {}, step: {:.2}", angle_span, count, step);
}

/// Approximates the projection of a line in R3 to R2.
///
/// The `resolution` and `max_split` arguments control how fine the
/// approximation is. If two projected points are farther than `resolution`
/// pixels apart, then midpoint of those two points in R3 is also projected.
/// This process will continue until the projected points are closer than
/// `resolution`, or until the line has been split `max_split` times.
///
/// If the split limit is hit, then instead of rendering line segments between the remaining points
pub fn approximate_curve(
    a: &R3,
    b: &R3,
    camera: Camera,
    resolution: f64,
    max_split: i32,
) -> Vec<[f64; 2]> {
    let mut done = Vec::new();
    let mut todo = Vec::new();

    done.push((*a, to_screen_space(a, &camera)));
    todo.push((*b, to_screen_space(b, &camera)));

    let mut branch_done = Vec::new();
    branch_done.push(false);

    let finish_branch = |branch_done: &mut Vec<bool>| {
        // finish up all the branches we are done with, and our branch
        while branch_done.pop().unwrap() {}
        // note that we are now done with our branch
        branch_done.push(true);
    };

    while let Some((end, [end_x, end_y])) = todo.last() {
        let (begin, [begin_x, begin_y]) = done.last().unwrap();

        let distance = ((begin_x - end_x).powi(2) + (begin_y - end_y).powi(2)).sqrt();

        if distance <= resolution {
            // we are done with this level
            done.push(todo.pop().unwrap());
            finish_branch(&mut branch_done);
        } else if branch_done.len() > max_split.try_into().unwrap() {
            // can't do any more splits, instead switch to approximating a circle if the points are behind us
            if is_behind(begin, &camera) && is_behind(end, &camera) {
                approximate_circle(*begin_x, *begin_y, *end_x, *end_y, |pos| { done.push((*end, pos)) });
            }
            done.push(todo.pop().unwrap());
            finish_branch(&mut branch_done);
        } else {
            // split
            let mid = midpoint(begin, end);
            todo.push((mid, to_screen_space(&mid, &camera)));
            branch_done.push(false);
        }
    }

    done.iter().map(|&x| x.1).collect()
}

pub fn render_curve(
    color: graphics::types::Color,
    points: &[[f64; 2]],
    debug: bool,
    c: &graphics::Context,
    g: &mut opengl_graphics::GlGraphics,
    center: graphics::math::Matrix2d,
) {
    match points.get(0) {
        None => return,

        Some(start) => {
            let line = graphics::Line::new(color, 1.0);
            let mut prev = start;
            for i in 1..points.len() {
                let next = &points[i];
                line.draw(
                    [prev[0], prev[1], next[0], next[1]],
                    &c.draw_state,
                    center,
                    g,
                );
                // debug dots
                if debug {
                    if i != points.len() - 1 {
                        graphics::Ellipse::new([1.0, 1.0, 1.0, 0.5]).draw(
                            graphics::ellipse::circle(0.0, 0.0, 2.0),
                            &c.draw_state,
                            center.trans(next[0], next[1]),
                            g,
                        );
                    }
                }
                prev = next;
            }
        }
    }
}

fn flush_graphics(transform: graphics::math::Matrix2d, g: &mut opengl_graphics::GlGraphics) {
    let color = [0.0, 0.0, 0.0, 1.0];
    let rect = [-540.0, -540.0, 1.0, 1.0];
    graphics::Rectangle::new(color).draw(rect, &graphics::DrawState::default(), transform, g);
    g.clear_draw_state();
    graphics::Rectangle::new(color).draw(rect, &graphics::DrawState::default(), transform, g);
}

pub fn draw_poly(
    color: graphics::types::Color,
    poly: &[[f64; 2]],
    is_behind: bool,
    _draw_state: &graphics::DrawState,
    transform: graphics::math::Matrix2d,
    g: &mut opengl_graphics::GlGraphics,
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
        graphics::Ellipse::new([1.0, 1.0, 1.0, 1.0]).draw(
            graphics::rectangle::square(-540.0, -540.0, 1080.0),
            &clip,
            transform,
            g,
        );
    }

    graphics::Rectangle::new(color).draw(
        [-540.0, -540.0, 1080.0, 1080.0],
        &graphics::DrawState::new_inside(),
        transform,
        g,
    );

    flush_graphics(transform, g);

    // debug points
    // for (n, p) in poly.iter().enumerate() {
    //     let shade = n as f32 / poly.len() as f32;
    //     graphics::Ellipse::new([1.0-shade, 1.0, shade, 1.0])
    //         .draw(graphics::ellipse::circle(0.0, 0.0, 2.0), draw_state, transform.trans(p[0], p[1]), g);
    // }
}

pub fn to_screen_space(point: &R3, camera: &Camera) -> [f64; 2] {
    let to_point = *point - camera.position;

    let forward = camera.orientation.rotate(&R3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    });
    let right = camera.orientation.rotate(&R3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    });

    let alpha = dot(&to_point.normalized(), &forward).acos();

    // Don't vom when at the poles
    if alpha == 0.0 {
        [0.0, 0.0]
    } else if alpha == std::f64::consts::PI {
        [camera.scale * alpha, 0.0]
    } else {
        let beta = alpha / (to_point - forward * dot(&to_point, &forward)).norm();
        let x = beta * dot(&to_point, &right);
        let y = beta * dot(&to_point, &cross(&forward, &right));
        [camera.scale * x, camera.scale * y]
    }
}
