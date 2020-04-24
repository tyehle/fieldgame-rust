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

pub fn render_curve(
    color: graphics::types::Color,
    points: &[[f64; 2]],
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
                if i != points.len() - 1 {
                    graphics::Ellipse::new([1.0, 1.0, 1.0, 0.5]).draw(
                        graphics::ellipse::circle(0.0, 0.0, 2.0),
                        &c.draw_state,
                        center.trans(next[0], next[1]),
                        g,
                    );
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
