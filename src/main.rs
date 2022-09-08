use std::time::Duration;
use std::time::Instant;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::{OpenGLWindow, WindowSettings};
// use std::time::SystemTime;

mod mesh;
mod r3;
use r3::*;
mod render;
use r3::quaternion::*;

pub struct GameObject {
    mesh: mesh::Mesh,
    pose: pose::Pose,

    velocity: R3,
    acceleration: R3,

    angular_velocity: R3,
    angular_acceleration: R3,
}

impl GameObject {
    fn physics_step(&mut self, dt: f64) {
        self.velocity += self.acceleration * dt;
        self.pose.pos += self.velocity * dt;

        self.angular_velocity += self.angular_acceleration * dt;
        // q_next = ( 1 + 1/2 * dt * angular_velocity ) * q
        // see https://gamedev.stackexchange.com/a/157018
        self.pose.orientation =
            Quaternion::from_real_imaginary(1.0, &(self.angular_velocity * 0.5 * dt))
                * self.pose.orientation;
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend
    glyph_cache: GlyphCache<'static>,

    prev_fps_update: Instant,
    frames_since_prev_fps: i32,
    fps: f64,

    // input
    control_magnitude: f64, // size of roll control input
    left: bool,             // input state
    right: bool,            // input state
    up: bool,               // input state
    down: bool,             // input state
    forward: bool,
    back: bool,
    draw_hud: bool,

    mouse_left: bool,
    mouse_right: bool,
    mouse_mov: [f64; 2],
    mouse_scroll: [f64; 2],

    // player state
    acceleration: f64,
    velocity: f64,
    camera: render::Camera,

    // game objects
    objects: Vec<GameObject>,
    // Game state
    debug: bool,
    // in_cube: bool,
    // score: i32,
    // last_score: SystemTime,
    // timeout_sec: u32,
}

fn initial_app(
    gl: GlGraphics,
    control_magnitude: f64,
    acceleration: f64,
    velocity: f64,
    // last_score: SystemTime,
    // timeout_sec: u32,
) -> App {
    fn cube(rotation: Quaternion) -> GameObject {
        let pose = pose::Pose {
            pos: R3::new(100.0 * (2.0 / 3.0_f64).sqrt() * 1.5, 0.0, 0.0),
            orientation: Quaternion::rotation(
                R3::new(1.0, -1.0, 0.0).normalized(),
                (1.0 / 3_f64.sqrt()).acos(),
            ),
        };

        GameObject {
            mesh: mesh::cuboid(R3::new(100.0, 100.0, 100.0), [0.5, 0.0, 0.5, 1.0]),
            pose: pose.rotate(R3::zero(), rotation),

            acceleration: R3::zero(),
            velocity: R3::zero(),

            angular_acceleration: rotation.rotate(&R3::new(0.0, 0.0, -0.0)),
            // angular_velocity: rotation.rotate(&R3::new(0.0, 0.0, -1.0)),
            angular_velocity: rotation.rotate(&R3::new(0.0, 0.0, 0.0))
        }
    }

    fn octahedron(rotation: Quaternion) -> GameObject {
        let pose = pose::Pose {
            pos: R3::new(0.0, 0.0, 0.0),
            orientation: Quaternion::zero_rotation(),
        };

        let mesh = mesh::mk_meshes("data/octahedron.obj", [0.0, 0.33, 0.67, 1.0]).unwrap();

        GameObject {
            mesh: mesh::scale(mesh, 0.25),
            pose: pose.rotate(R3::zero(), rotation),

            acceleration: R3::zero(),
            velocity: R3::zero(),

            angular_acceleration: rotation.rotate(&R3::new(0.0, 0.0, -0.0)),
            angular_velocity: R3::zero(),
            // angular_velocity: rotation.rotate(&R3::new(0.0, 0.0, -1.0)),
        }
    }

    fn diamond(rotation: Quaternion) -> GameObject {
        let pose = pose::Pose {
            pos: R3::new(100.0 * (2.0 / 3.0_f64).sqrt() * 1.5, 0.0, 0.0),
            orientation: Quaternion::zero_rotation(),
        };

        let mesh = mesh::mk_meshes("data/diamond.obj", [0.0, 0.5, 0.5, 1.0]).unwrap();

        GameObject {
            mesh,
            pose: pose.rotate(R3::zero(), rotation),

            acceleration: R3::zero(),
            velocity: R3::zero(),

            angular_acceleration: rotation.rotate(&R3::new(0.0, 0.0, -0.0)),
            angular_velocity: rotation.rotate(&R3::new(0.0, 0.0, -1.0)),
        }
    }

    fn teapot(rotation: Quaternion) -> GameObject {
        let pose = pose::Pose {
            pos: R3::new(5.0, 0.0, 2.0),
            orientation: Quaternion::rotation(R3::new(0.0, 0.0, -1.0), 0.5 * core::f64::consts::PI)
                * Quaternion::rotation(R3::new(-1.0, 0.0, 0.0), 0.5 * core::f64::consts::PI),
        };

        let mesh = mesh::mk_meshes("data/teapot.obj", [0.0, 0.5, 0.5, 1.0]).unwrap();

        GameObject {
            mesh: mesh::condense_mesh(&mesh),
            pose: pose.rotate(R3::zero(), rotation),

            acceleration: R3::zero(),
            velocity: R3::zero(),

            angular_acceleration: rotation.rotate(&R3::new(0.0, 0.0, -0.0)),
            angular_velocity: rotation.rotate(&R3::new(0.0, 0.0, -1.0)),
        }
    }

    fn ship(rotation: Quaternion) -> GameObject {
        let pose = pose::Pose {
            pos: R3::new(0.0, 0.0, 0.0),
            orientation: Quaternion::rotation(R3::new(0.0, 0.0, -1.0), 0.5 * core::f64::consts::PI),
        };

        let mesh = mesh::mk_meshes("models/hole-ship-wire.obj", [0.0, 0.5, 0.5, 1.0]).unwrap();
        // println!("{:?}", mesh::condense_mesh(&mesh));

        GameObject {
            mesh: mesh::condense_mesh(&mesh),
            pose: pose.rotate(R3::zero(), rotation),

            acceleration: R3::zero(),
            velocity: R3::zero(),

            angular_acceleration: rotation.rotate(&R3::new(0.0, 0.0, 0.0)),
            angular_velocity: rotation.rotate(&R3::new(0.0, 0.0, -0.25)),
        }
    }

    let camera = render::Camera {
        position: R3::new(-30.0, 0.0, -30.0),
        orientation: Quaternion::rotation(R3::new(0.0, -1.0, 0.0), 0.25 * core::f64::consts::PI),
        scale: 1080.0 / std::f64::consts::PI / 2.0,
    };

    App {
        gl,
        glyph_cache: GlyphCache::new("OpenSans-Regular.ttf", (), TextureSettings::new()).unwrap(),

        prev_fps_update: Instant::now(),
        frames_since_prev_fps: 0,
        fps: 0.0,

        control_magnitude,
        left: false,
        right: false,
        up: false,
        down: false,
        forward: false,
        back: false,
        draw_hud: true,

        mouse_left: false,
        mouse_right: false,
        mouse_mov: [0.0, 0.0],
        mouse_scroll: [0.0, 0.0],

        acceleration,
        velocity,
        camera,

        objects: vec![
            // cube(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), 0.0 * core::f64::consts::PI)),
            // cube(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), (2.0/3.0) * core::f64::consts::PI)),
            // cube(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), -(2.0/3.0) * core::f64::consts::PI)),

            // teapot(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), 0.0 * core::f64::consts::PI)),
            // teapot(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), (2.0/3.0) * core::f64::consts::PI)),
            // teapot(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), -(2.0/3.0) * core::f64::consts::PI)),

            // diamond(Quaternion::rotation(R3::new(0.0, 1.0, 0.0), -(2.0/3.0) * core::f64::consts::PI)),

            // cube(Quaternion::zero_rotation()),
            octahedron(Quaternion::zero_rotation()),
            // ship(Quaternion::zero_rotation()),
        ],
        debug: false,
        // in_cube: false,
        // score: 0,
        // last_score,
        // timeout_sec,
    }
}

impl App {
    fn render(&mut self, args: RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        // const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.5, 1.0, 1.0];
        // const OUT:   [f32; 4] = [0.5, 0.0, 0.5, 1.0];
        // const IN:    [f32; 4] = [0.0, 0.25, 0.5, 1.0];

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        let camera = self.camera;
        let draw_hud = self.draw_hud;
        let objects = &self.objects;
        let glyph_cache = &mut self.glyph_cache;
        let fps = self.fps;
        let debug = self.debug;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            for obj in objects {
                mesh::render_mesh(
                    &obj.mesh,
                    &obj.pose,
                    debug,
                    &c,
                    gl,
                    camera,
                    c.transform.trans(x, y),
                );
            }

            if draw_hud {
                // render some HUD stuff
                // TODO: Figure out how big the screen actually is
                Line::new(BLUE, 1.0).draw(
                    [0.0, -5.0, 0.0, 5.0],
                    &c.draw_state,
                    c.transform.trans(x, y),
                    gl,
                );
                Line::new(BLUE, 1.0).draw(
                    [-5.0, 0.0, 5.0, 0.0],
                    &c.draw_state,
                    c.transform.trans(x, y),
                    gl,
                );
                Ellipse::new_border(BLUE, 0.5).draw(
                    rectangle::square(-269.5, -269.5, 539.0),
                    &c.draw_state,
                    c.transform.trans(x, y),
                    gl,
                );
                Ellipse::new_border(BLUE, 1.0).draw(
                    rectangle::square(-539.0, -539.0, 1078.0),
                    &c.draw_state,
                    c.transform.trans(x, y),
                    gl,
                );

                let info = format!(
                    "FPS: {:.2}\ncamera position: ({:.2}, {:.2}, {:.2})",
                    fps, camera.position.x, camera.position.y, camera.position.z
                );

                for (i, line) in info.lines().enumerate() {
                    Text::new_color(BLUE, 14)
                        .draw(
                            line,
                            glyph_cache,
                            &c.draw_state,
                            c.transform.trans(10.0, 21.0 * (i as f64) + 24.0),
                            gl,
                        )
                        .unwrap();
                }
            }
        });
    }

    fn update(&mut self, args: UpdateArgs) {
        const FORWARD: R3 = R3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };

        const RIGHT: R3 = R3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        // move the camera with the mouse
        if self.mouse_right && self.mouse_mov != [0.0, 0.0] {
            let speed = 0.01;

            let angular_velocity = R3::new(0.0, -self.mouse_mov[1], self.mouse_mov[0]) * speed;
            let axis = self
                .camera
                .orientation
                .rotate(&angular_velocity.normalized());
            let angle = angular_velocity.norm();
            let rotation = Quaternion::rotation(axis, angle);

            self.camera.position = rotation.rotate(&self.camera.position);
            self.camera.orientation = rotation * self.camera.orientation;
        }
        self.mouse_mov = [0.0, 0.0];

        if self.mouse_scroll[1] != 0.0 {
            let distance = self.camera.position.norm();
            let speed = 0.05;

            let velocity = self.mouse_scroll[1] * distance * speed;

            self.camera.position += self.camera.orientation.rotate(&R3::new(velocity, 0.0, 0.0));
        }
        self.mouse_scroll = [0.0, 0.0];

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
        let o1 = self.camera.orientation * Quaternion::rotation(RIGHT, pitch_rate * args.dt);

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
        let orientation = o1 * Quaternion::rotation(FORWARD, roll_rate * args.dt);

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
            position: self.camera.position + forward * self.velocity * args.dt,
            orientation,
            scale: self.camera.scale,
        };

        for obj in self.objects.iter_mut() {
            obj.physics_step(args.dt);
        }

        update_fps(self);

        // let was_inside = self.in_cube;
        // self.in_cube = inside(
        //     &R3 {
        //         x: 0.0,
        //         y: 0.0,
        //         z: 0.0,
        //     },
        //     &R3 {
        //         x: 100.0,
        //         y: 100.0,
        //         z: 100.0,
        //     },
        //     &self.camera.position,
        // );
        // if was_inside && !self.in_cube {
        //     self.velocity += self.acceleration * 4.0;
        //     // self.camera.position = self.camera.position + R3{x: 200.0, y: 0.0, z: 0.0};
        // }
    }

    fn button(&mut self, args: ButtonArgs) {
        let pressed = match args.state {
            ButtonState::Press => true,
            ButtonState::Release => false,
        };

        match args.button {
            Button::Mouse(MouseButton::Left) => self.mouse_left = pressed,
            Button::Mouse(MouseButton::Right) => self.mouse_right = pressed,

            Button::Keyboard(Key::D) => self.right = pressed,
            Button::Keyboard(Key::A) => self.left = pressed,
            Button::Keyboard(Key::W) => self.forward = pressed,
            Button::Keyboard(Key::S) => self.back = pressed,
            Button::Keyboard(Key::Space) => self.up = pressed,
            Button::Keyboard(Key::C) => self.down = pressed,
            Button::Keyboard(Key::H) => {
                if pressed {
                    self.draw_hud = !self.draw_hud;
                }
            }
            Button::Keyboard(Key::X) => {
                if pressed {
                    self.velocity = 0.0;
                }
            }
            Button::Keyboard(Key::P) => {
                if pressed {
                    self.debug = !self.debug;
                }
            }
            // Button::Keyboard(Key::LShift) => {},
            _ => {}
        }
    }

    fn mouse(&mut self, args: Motion) {
        match args {
            // Motion::MouseCursor([a, b]) => dbg!(args),
            Motion::MouseRelative(mov) => {
                self.mouse_mov[0] += mov[0];
                self.mouse_mov[1] += mov[1];
            }

            Motion::MouseScroll(mov) => {
                self.mouse_scroll[0] += mov[0];
                self.mouse_scroll[1] += mov[1];
            }

            _ => {}
        };
    }
}

/// Update the fps stats stored in app
fn update_fps(app: &mut App) {
    const MIN_FPS_UPDATE_INTERVAL_SECONDS: Duration = Duration::new(1, 0);

    app.frames_since_prev_fps += 1;

    let duration = app.prev_fps_update.elapsed();
    if duration >= MIN_FPS_UPDATE_INTERVAL_SECONDS {
        // time to update the fps counter
        app.prev_fps_update = Instant::now();
        app.fps = app.frames_since_prev_fps as f64 / duration.as_secs_f64();
        app.frames_since_prev_fps = 0;
    }
}

// fn inside(corner: &R3, size: &R3, pos: &R3) -> bool {
//     pos.x > corner.x
//         && pos.x < corner.x + size.x
//         && pos.y > corner.y
//         && pos.y < corner.y + size.y
//         && pos.z > corner.z
//         && pos.z < corner.z + size.z
// }

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V4_5;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [800, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .fullscreen(true)
        .vsync(true)
        // .samples(4)
        .build()
        .unwrap();

    // init the opengl function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let mut app = initial_app(
        GlGraphics::new(opengl),
        1.0,
        40.0,
        0.0,
        // SystemTime::now(),
        // 10,
    );

    let mut events = Events::new(EventSettings::new().max_fps(60).ups(60));
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Loop(Loop::Render(args)) => app.render(args),
            Event::Loop(Loop::Update(args)) => app.update(args),
            Event::Input(Input::Button(args), _) => app.button(args),
            Event::Input(Input::Move(args), _) => app.mouse(args),
            _ => {}
        }
    }
}
