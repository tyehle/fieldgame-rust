extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

mod render;

pub struct App {
    gl: GlGraphics,  // OpenGL drawing backend
    control_magnitude: f64,     // size of roll control input
    left: bool,      // input state
    right: bool,     // input state
    up: bool,        // input state
    down: bool,      // input state
    forward: bool,
    back: bool,
    camera: render::Camera
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

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            cube.render(&c, gl, camera, c.transform.trans(x, y));

            // render some HUD stuff
            Line::new(BLUE, 1.0).draw([0.0, -5.0, 0.0, 5.0], &c.draw_state, c.transform.trans(x, y), gl);
            Line::new(BLUE, 1.0).draw([-5.0, 0.0, 5.0, 0.0], &c.draw_state, c.transform.trans(x, y), gl);
            Ellipse::new_border(BLUE, 1.0).draw(rectangle::square(-540.0, -540.0, 1080.0), &c.draw_state, c.transform.trans(x, y), gl);

            // let transform = c.transform.trans(x, y)
            //                            .rot_rad(rotation)
            //                            .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            // rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let x = {
            if self.forward && !self.back {
                self.control_magnitude
            } else if !self.forward && self.back {
                -self.control_magnitude
            } else {
                0.0
            }
        };

        let y = {
            if self.right && !self.left {
                self.control_magnitude
            } else if !self.right && self.left {
                -self.control_magnitude
            } else {
                0.0
            }
        };

        let z = {
            if self.up && !self.down {
                -self.control_magnitude
            } else if !self.up && self.down {
                self.control_magnitude
            } else {
                0.0
            }
        };

        // update camera position
        let velocity = render::R3{x: x, y: y, z: z};
        self.camera = render::Camera {
            position: self.camera.position + velocity*args.dt,
            forward: self.camera.forward,
            right: self.camera.right,
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

    let mut app = App {
        gl: GlGraphics::new(opengl),
        control_magnitude: 200.0,
        left: false,
        right: false,
        up: false,
        down: false,
        forward: false,
        back: false,
        camera: render::Camera {
            position: render::R3 {x: 50.0, y: 50.0, z: 50.0},
            forward: render::R3 {x: 1.0, y: 0.0, z: 0.0},
            right: render::R3 {x: 0.0, y: 1.0, z: 0.0},
            scale: 1080.0 / 3.14 / 2.0
        }
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
