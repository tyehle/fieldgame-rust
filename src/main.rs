// extern crate piston;
// extern crate graphics;
// extern crate glutin_window;
// extern crate opengl_graphics;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::event_loop::*;
use piston::input::*;
use piston::window::{WindowSettings, OpenGLWindow};
use std::time::SystemTime;

mod cuboid;
use cuboid::*;
mod face;
mod mesh;
mod r3;
use r3::*;
mod render;
use render::Renderable;
use r3::quaternion::*;

pub struct App {
    gl: GlGraphics,  // OpenGL drawing backend

    // input
    control_magnitude: f64,     // size of roll control input
    left: bool,      // input state
    right: bool,     // input state
    up: bool,        // input state
    down: bool,      // input state
    forward: bool,
    back: bool,
    draw_hud: bool,

    // player state
    acceleration: f64,
    velocity: f64,
    camera: render::Camera,

    // game objects
    mesh: mesh::Mesh,

    // Game state
    in_cube: bool,
    score: i32,
    last_score: SystemTime,
    timeout_sec: u32,
}

fn initial_app(gl: GlGraphics, control_magnitude: f64, acceleration: f64, velocity: f64, camera: render::Camera, last_score: SystemTime, timeout_sec: u32) -> App {
    App {
        gl,

        control_magnitude,
        left: false,
        right: false,
        up: false,
        down: false,
        forward: false,
        back: false,
        draw_hud: true,

        acceleration,
        velocity,
        camera,

        mesh: mesh::cuboid(R3::new(100.0, 100.0, 100.0), [0.5, 0.0, 0.5, 1.0]),

        in_cube: false,
        score: 0,
        last_score,
        timeout_sec,
    }
}

impl App {
    fn render(&mut self, args: RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        // const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE:  [f32; 4] = [0.0, 0.5, 1.0, 1.0];
        const OUT:   [f32; 4] = [0.5, 0.0, 0.5, 1.0];
        const IN:    [f32; 4] = [0.0, 0.25, 0.5, 1.0];


        let cube = Cuboid {
            position: R3 {x: 0.0, y: 0.0, z: 0.0},
            velocity: R3 {x: 0.0, y: 0.0, z: 0.0},
            size: R3 {x: 100.0, y: 100.0, z: 100.0},
            color: if self.in_cube {IN} else {OUT},
         };


        // let square = rectangle::square(0.0, 0.0, 50.0);
        // let rotation = self.roll_x;
        let (x, y) = (args.window_size[0] / 2.0,
                      args.window_size[1] / 2.0);
        let camera = self.camera;
        let draw_hud = self.draw_hud;
        let mesh = &self.mesh;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            // cube.render(&c, gl, camera, c.transform.trans(x, y));

            let zero = R3::new(0.0, 0.0, 0.0);
            let orientation = quaternion::rotation(R3::new(1.0, 1.0, 1.0), std::f64::consts::PI * 0.25);
            mesh::render_mesh(mesh, &pose::Pose{pos: zero, orientation}, &c, gl, camera, c.transform.trans(x, y));

            if draw_hud {
                // render some HUD stuff
                // TODO: Figure out how big the screen actually is
                Line::new(BLUE, 1.0).draw([0.0, -5.0, 0.0, 5.0], &c.draw_state, c.transform.trans(x, y), gl);
                Line::new(BLUE, 1.0).draw([-5.0, 0.0, 5.0, 0.0], &c.draw_state, c.transform.trans(x, y), gl);
                Ellipse::new_border(BLUE, 0.5).draw(rectangle::square(-270.0, -270.0, 540.0), &c.draw_state, c.transform.trans(x, y), gl);
                Ellipse::new_border(BLUE, 1.0).draw(rectangle::square(-540.0, -540.0, 1080.0), &c.draw_state, c.transform.trans(x, y), gl);
            }

            // let transform = c.transform.trans(x, y)
            //                            .rot_rad(rotation)
            //                            .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            // rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: UpdateArgs) {
        // pitch
        let pitch_rate = {
            if self.forward && !self.back {
                -self.control_magnitude
            } else if !self.forward && self.back {
                self.control_magnitude
            } else {
                0.0
            }
        };
        const RIGHT: R3 = R3 { x: 0.0, y: 1.0, z: 0.0 };
        let o1 = self.camera.orientation * rotation(RIGHT, pitch_rate * args.dt);

        // roll
        let roll_rate = {
            if self.right && !self.left {
                -self.control_magnitude
            } else if !self.right && self.left {
                self.control_magnitude
            } else {
                0.0
            }
        };
        // rotate around the new forward vector to keep them orthogonal
        const FORWARD: R3 = R3 { x: 1.0, y: 0.0, z: 0.0 };
        let orientation = o1 * rotation(FORWARD, roll_rate * args.dt);

        // speed
        let a = {
            if self.up && !self.down {
                -self.acceleration
            } else if !self.up && self.down {
                self.acceleration
            } else {
                0.0
            }
        };
        self.velocity += a * args.dt;

        let forward = orientation.rotate(&FORWARD);

        self.camera = render::Camera {
            position: self.camera.position + forward*self.velocity*args.dt,
            orientation,
            scale: self.camera.scale
        };

        let was_inside = self.in_cube;
        self.in_cube = inside(&R3{x: 0.0, y: 0.0, z: 0.0}, &R3{x: 100.0, y: 100.0, z: 100.0}, &self.camera.position);
        if was_inside && !self.in_cube {
            self.velocity += self.acceleration * 4.0;
            // self.camera.position = self.camera.position + R3{x: 200.0, y: 0.0, z: 0.0};
        }
    }

    fn button(&mut self, args: ButtonArgs) {
        let pressed = match args.state {
            ButtonState::Press => true,
            ButtonState::Release => false
        };

        match args.button {
            Button::Keyboard(Key::D) => self.right = pressed,
            Button::Keyboard(Key::A) => self.left = pressed,
            Button::Keyboard(Key::W) => self.forward = pressed,
            Button::Keyboard(Key::S) => self.back = pressed,
            Button::Keyboard(Key::Space) => self.up = pressed,
            Button::Keyboard(Key::C) => self.down = pressed,
            Button::Keyboard(Key::H) => if pressed { self.draw_hud = !self.draw_hud; }
            Button::Keyboard(Key::X) => if pressed { self.velocity = 0.0; }
            // Button::Keyboard(Key::LShift) => {},
            _ => {}
        }
    }
}

fn inside(corner: &R3, size: &R3, pos: &R3) -> bool {
    pos.x > corner.x && pos.x < corner.x+size.x
        && pos.y > corner.y && pos.y < corner.y+size.y
        && pos.z > corner.z && pos.z < corner.z+size.z
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new(
            "spinning-square",
            [800, 600]
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .fullscreen(true)
        .vsync(true)
        .build()
        .unwrap();

    // init the opengl function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let camera = render::Camera {
        position: R3 { x: 50.0, y: 50.0, z: 50.0 },
        orientation: Quaternion { r: 1.0, i: 0.0, j: 0.0, k: 0.0 },
        scale: 1080.0 / std::f64::consts::PI / 2.0
    };

    let mut app = initial_app(GlGraphics::new(opengl), 1.0, 40.0, 20.0, camera, SystemTime::now(), 10);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Loop(Loop::Render(args)) => app.render(args),
            Event::Loop(Loop::Update(args)) => app.update(args),
            Event::Input(Input::Button(args), _) => app.button(args),
            _ => {}
        }
    }
}
