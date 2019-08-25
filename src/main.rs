extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

pub struct App {
    gl: GlGraphics,  // OpenGL drawing backend
    roll_v_max: f64, // Max roll rate
    roll_u: f64,     // size of roll control input
    left: bool,      // input state
    right: bool,     // input state
    roll_x: f64,     // Roll position
    roll_v: f64,     // Roll rate
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.roll_x;
        let (x, y) = (args.window_size[0] / 2.0,
                      args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // find acceleration
        let mut a = 0.0;
        if self.right { a += self.roll_u }
        if self.left { a -= self.roll_u }

        // update velocity
        self.roll_v += a * args.dt;
        // limit max roll rate
        if self.roll_v > self.roll_v_max {
            self.roll_v = self.roll_v_max;
        } else if self.roll_v < -self.roll_v_max {
            self.roll_v = -self.roll_v_max;
        }

        // update position
        self.roll_x += self.roll_v * args.dt;
    }

    fn press(&mut self, args: Button) {
        match args {
            Button::Keyboard(Key::Right) => self.right = true,
            Button::Keyboard(Key::Left) => self.left = true,
            _ => {}
        }
    }

    fn release(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::Right) => self.right = false,
            Button::Keyboard(Key::Left) => self.left = false,
            _ => {}
        }
    }
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

    let mut app = App {
        gl: GlGraphics::new(opengl),
        roll_v_max: 2.0,
        roll_u: 2.0,
        left: false,
        right: false,
        roll_x: 0.0,
        roll_v: 0.0,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        e.render(|x| {app.render(x)});
        e.update(|x| {app.update(x)});
        e.press(|x| {app.press(x)});
        e.release(|x| {app.release(x)});
    }
}
