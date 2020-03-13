// extern crate piston;
// extern crate graphics;
// extern crate glutin_window;
// extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::window::OpenGLWindow;

mod render;
mod quaternion;

pub struct App {
    gl: GlGraphics,  // OpenGL drawing backend
    control_magnitude: f64,     // size of roll control input
    acceleration: f64,
    left: bool,      // input state
    right: bool,     // input state
    up: bool,        // input state
    down: bool,      // input state
    forward: bool,
    back: bool,
    velocity: f64,
    camera: render::Camera,
    draw_hud: bool
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        use render::Renderable;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE:  [f32; 4] = [0.0, 0.5, 1.0, 1.0];


        let cube = render::Cube {
            position: render::R3 {x: 0.0, y: 0.0, z: 0.0},
            velocity: render::R3 {x: 0.0, y: 0.0, z: 0.0},
            size: render::R3 {x: 100.0, y: 100.0, z: 100.0},
            color: RED
         };


        // let square = rectangle::square(0.0, 0.0, 50.0);
        // let rotation = self.roll_x;
        let (x, y) = (args.window_size[0] / 2.0,
                      args.window_size[1] / 2.0);
        let camera = self.camera;
        let draw_hud = self.draw_hud;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            cube.render(&c, gl, camera, c.transform.trans(x, y));

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

    fn update(&mut self, args: &UpdateArgs) {
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
        let forward = self.camera.forward.rotate(pitch_rate*args.dt, self.camera.right);

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
        let right = self.camera.right.rotate(roll_rate*args.dt, forward);

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
        self.velocity = self.velocity + a*args.dt;

        self.camera = render::Camera {
            position: self.camera.position + forward*self.velocity*args.dt,
            forward: forward,
            right: right,
            scale: self.camera.scale
        }
    }

    fn button(&mut self, args: &ButtonArgs) {
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
            Button::Keyboard(Key::LShift) => {},
            _ => {}
        }
    }
}

fn main() {
    // let (x, y) = render::to_screen_space(
    //     render::R3 {
    //         x: 2.0,
    //         y: 0.1,
    //         z: 0.0,
    //     },
    //     &render::Camera {
    //         position: render::R3 { x: 0.0, y: 0.0, z: 0.0 },
    //         forward: render::R3 { x: 1.0, y: 0.0, z: 0.0 },
    //         right: render::R3 { x: 0.0, y: 1.0, z: 0.0 },
    //         scale: 1.0,
    //     }
    // );
    // println!("({}, {})", x, y);

    // return;

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

    let mut app = App {
        gl: GlGraphics::new(opengl),
        control_magnitude: 1.0,
        acceleration: 40.0,
        left: false,
        right: false,
        up: false,
        down: false,
        forward: false,
        back: false,
        velocity: 20.0,
        camera: render::Camera {
            position: render::R3 {x: 50.0, y: 50.0, z: 50.0},
            forward: render::R3 {x: 1.0, y: 0.0, z: 0.0},
            right: render::R3 {x: 0.0, y: 1.0, z: 0.0},
            scale: 1080.0 / 3.14 / 2.0
        },
        draw_hud: true
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Loop(Loop::Render(args)) => app.render(&args),
            Event::Loop(Loop::Update(args)) => app.update(&args),
            Event::Input(Input::Button(args), _) => app.button(&args),
            _ => {}
        }
    }
}
