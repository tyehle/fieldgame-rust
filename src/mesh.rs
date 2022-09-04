use graphics;
use std::collections::HashMap;
use std::fs;
use wavefront_obj::obj;

use graphics::types::Color;

use super::r3::{cross, dot, pose::Pose, R3};
use super::render::*;

pub fn condense_mesh(mesh: &Mesh) -> Mesh {
    let mut mapping = Vec::new();
    let mut vertices = Vec::new();

    for &v in &mesh.vertices {
        match vertices.iter().position(|&nv| nv == v) {
            Some(index) => {
                mapping.push(index);
            }

            None => {
                mapping.push(vertices.len());
                vertices.push(v);
            }
        }
    }

    Mesh {
        vertices,
        edges: mesh
            .edges
            .iter()
            .map(|&(a, b)| (mapping[a], mapping[b]))
            .collect(),

        lines: mesh.lines.clone(),
        triangles: mesh.triangles.clone(),
        parallelograms: mesh.parallelograms.clone(),
    }
}

pub fn mk_meshes(path: &str, color: Color) -> Result<Mesh, String> {
    let file = fs::read_to_string(path).map_err(|_| "Could not read file")?;

    let obj_set = obj::parse(file).map_err(|e| e.message)?;

    let mut vertices = Vec::new();
    let mut vertex_offset;

    let mut edge_map = HashMap::new();
    let mut edges = Vec::new();

    fn get_edge(
        edges: &mut Vec<(usize, usize)>,
        edge_map: &mut HashMap<(usize, usize), usize>,
        a: usize,
        b: usize,
    ) -> usize {
        if b < a {
            get_edge(edges, edge_map, b, a)
        } else {
            // add this edge to the list if its not already there
            match edge_map.get(&(a, b)) {
                Some(&index) => index,

                None => {
                    let index = edges.len();
                    edges.push((a, b));
                    edge_map.insert((a, b), index);
                    index
                }
            }
        }
    }

    let mut lines = Vec::new();
    let mut triangles = Vec::new();
    let face_color = [color[0], color[1], color[2], 0.125 * color[3]];

    for object in &obj_set.objects {
        vertex_offset = vertices.len();
        vertices.extend(object.vertices.iter().map(|v| R3::new(v.x, v.y, v.z)));

        for g in &object.geometry {
            for shape in &g.shapes {
                match shape.primitive {
                    obj::Primitive::Point(p) => println!("Ignoring a point! {}", p.0),

                    obj::Primitive::Line((obj_a, _, _an), (obj_b, _, _bn)) => {
                        let a = obj_a + vertex_offset;
                        let b = obj_b + vertex_offset;
                        lines.push((get_edge(&mut edges, &mut edge_map, a, b), color));
                    }

                    obj::Primitive::Triangle((obj_a, _, _an), (obj_b, _, _bn), (obj_c, _, _cn)) => {
                        let a = obj_a + vertex_offset;
                        let b = obj_b + vertex_offset;
                        let c = obj_c + vertex_offset;
                        // println!("T <{}, {}, {}>", a, b, c);
                        let ab = get_edge(&mut edges, &mut edge_map, a, b);
                        let bc = get_edge(&mut edges, &mut edge_map, b, c);
                        let ca = get_edge(&mut edges, &mut edge_map, c, a);
                        // lines.push((ab, color));
                        // lines.push((bc, color));
                        // lines.push((ca, color));
                        triangles.push((
                            [
                                (ab, edges[ab].0 != a),
                                (bc, edges[bc].0 != b),
                                (ca, edges[ca].0 != c),
                            ],
                            face_color,
                        ));
                    }
                }
            }
        }
    }

    Ok(Mesh {
        vertices,
        edges,
        lines,
        triangles,
        parallelograms: Vec::new(),
    })
}

#[derive(Debug)]
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
    debug: bool,
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
        render_curve(*color, &curves[*ci], debug, context, g, center);
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
