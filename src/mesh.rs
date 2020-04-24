use graphics;

use graphics::types::Color;

use super::r3::{cross, dot, pose::Pose, R3};
use super::render::*;

pub struct Mesh {
    pub vertices: Vec<R3>,
    pub edges: Vec<(usize, usize)>,
    pub lines: Vec<(usize, Color)>,
    pub triangles: Vec<([(usize, bool); 3], Color)>,
    pub parallelograms: Vec<([(usize, bool); 4], Color)>,
}

/// A cuboid mesh with a given size and color
pub fn cuboid(size: R3, color: Color) -> Mesh {
    let half_size = size * 0.5;

    let vertices = vec![
        R3::new(half_size.x, half_size.y, half_size.z),
        R3::new(half_size.x, half_size.y, -half_size.z),
        R3::new(half_size.x, -half_size.y, -half_size.z),
        R3::new(half_size.x, -half_size.y, half_size.z),
        R3::new(-half_size.x, -half_size.y, half_size.z),
        R3::new(-half_size.x, -half_size.y, -half_size.z),
        R3::new(-half_size.x, half_size.y, -half_size.z),
        R3::new(-half_size.x, half_size.y, half_size.z),
    ];

    let edges = vec![
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 0),
        (0, 3),
        (1, 6),
        (2, 5),
        (4, 7),
    ];

    let lines = (0..edges.len()).map(|i| (i, color)).collect();

    let face_color = [color[0], color[1], color[2], 0.25 * color[3]];
    let parallelograms = vec![
        ([(0, false), (1, false), (2, false), (8, true)], face_color),
        ([(0, false), (9, false), (6, false), (7, false)], face_color),
        ([(1, false), (10, false), (5, false), (9, true)], face_color),
        ([(2, false), (3, false), (4, false), (10, true)], face_color),
        (
            [(3, false), (11, false), (7, false), (8, false)],
            face_color,
        ),
        ([(4, false), (5, false), (6, false), (11, true)], face_color),
    ];

    Mesh {
        vertices,
        edges,
        lines,
        triangles: vec![],
        parallelograms,
    }
}

pub fn intersects_parallelogram(origin: &R3, direction: &R3, face: &[R3; 4]) -> bool {
    let [a, b, _, c] = *face;

    let normal = cross(&(a - b), &(a - c));
    let ao = a - *origin;
    let m = cross(direction, &ao);

    // divides are much more expensive than multiplies, so only do it once here
    let invdet = 1.0 / dot(direction, &normal);

    let t = dot(&ao, &normal) * invdet;
    let u = dot(&(a - c), &m) * invdet;
    let v = -dot(&(a - b), &m) * invdet;

    t >= 0.0 && u >= 0.0 && v >= 0.0 && u <= 1.0 && v <= 1.0
}

pub fn intersects_triangle(origin: &R3, direction: &R3, face: &[R3; 3]) -> bool {
    let [a, b, c] = *face;

    let normal = cross(&(a - b), &(a - c));
    let ao = a - *origin;
    let m = cross(direction, &ao);

    // divides are much more expensive than multiplies, so only do it once here
    let invdet = 1.0 / dot(direction, &normal);

    let t = dot(&ao, &normal) * invdet;
    let u = dot(&(a - c), &m) * invdet;
    let v = -dot(&(a - b), &m) * invdet;

    t >= 0.0 && u >= 0.0 && v >= 0.0 && u + v <= 1.0
}

pub fn render_mesh(
    mesh: &Mesh,
    pose: &Pose,
    context: &graphics::Context,
    g: &mut opengl_graphics::GlGraphics,
    camera: Camera,
    center: graphics::math::Matrix2d,
) {
    const RESOLUTION: f64 = 40.0;
    const MAX_SPLIT: i32 = 9;

    let transformed_vertices = mesh
        .vertices
        .iter()
        .map(|v| pose.orientation.rotate(v) + pose.pos)
        .collect::<Vec<_>>();

    let curves = mesh
        .edges
        .iter()
        .map(|(ai, bi)| {
            approximate_curve(
                &transformed_vertices[*ai],
                &transformed_vertices[*bi],
                camera,
                RESOLUTION,
                MAX_SPLIT,
            )
        })
        .collect::<Vec<_>>();

    for (ci, color) in &mesh.lines {
        render_curve(*color, &curves[*ci], context, g, center);
    }

    let backward = camera.orientation.rotate(&R3 {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    });

    fn map3<A, B>(xs: [A; 3], f: impl Fn(A) -> B) -> [B; 3] {
        let [a, b, c] = xs;
        [f(a), f(b), f(c)]
    }
    for &(edge_indices, color) in &mesh.triangles {
        let vs = map3(edge_indices, |(ei, rev)| {
            transformed_vertices[if rev {
                mesh.edges[ei].1
            } else {
                mesh.edges[ei].0
            }]
        });
        let is_behind = intersects_triangle(&camera.position, &backward, &vs);

        let mut points = Vec::new();
        for &(ci, rev) in &edge_indices {
            if rev {
                points.extend(curves[ci].iter().rev());
            } else {
                points.extend(&curves[ci]);
            }
        }

        draw_poly(color, &points, is_behind, &context.draw_state, center, g);
    }

    fn map4<A, B>(xs: [A; 4], f: impl Fn(A) -> B) -> [B; 4] {
        let [a, b, c, d] = xs;
        [f(a), f(b), f(c), f(d)]
    }
    for &(edge_indices, color) in &mesh.parallelograms {
        let vs = map4(edge_indices, |(ei, rev)| {
            transformed_vertices[if rev {
                mesh.edges[ei].1
            } else {
                mesh.edges[ei].0
            }]
        });
        let is_behind = intersects_parallelogram(&camera.position, &backward, &vs);

        let mut points = Vec::new();
        for &(ci, rev) in &edge_indices {
            if rev {
                points.extend(curves[ci].iter().rev());
            } else {
                points.extend(&curves[ci]);
            }
        }

        draw_poly(color, &points, is_behind, &context.draw_state, center, g);
    }
}
