use crate::piston::Window;
use glutin_window::GlutinWindow;
use graphics::draw_state::DrawState;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use std::path::Path;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
}
impl App {
    fn render(
        &mut self,
        args: &RenderArgs,
        image: &Image,
        texture: &Texture,
        &rect: &rectangle::Rectangle,
        &initial_cursor_pos: &[f64; 2],
        &cursor_pos: &[f64; 2],
        &initial_cursor_pos_set: &bool,
        &draw_state: &DrawState,
    ) {
        self.gl.draw(args.viewport(), |c, gl| {
            image.draw(texture, &draw_state, c.transform, gl);
            if !initial_cursor_pos_set {
                return;
            };
            rect.draw(
                [
                    initial_cursor_pos[0],
                    initial_cursor_pos[1],
                    cursor_pos[0] - initial_cursor_pos[0],
                    cursor_pos[1] - initial_cursor_pos[1],
                ],
                &draw_state,
                c.transform,
                gl,
            );
        });
    }
}

pub fn get_region(screenshot_path: &str) -> Option<[f64; 4]> {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: GlutinWindow = WindowSettings::new("sampic", [200, 200])
        .fullscreen(true)
        .decorated(false)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .samples(2)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let texture_settings = TextureSettings::new();
    let image = Image::new();
    //A texture to use with the image
    let texture = Texture::from_path(
        Path::new(&screenshot_path),
        &texture_settings,
    )
    .unwrap();
    let draw_state = &DrawState::new_alpha();

    let mut events = Events::new(EventSettings::new());
    let mut initial_cursor_pos = [0.0, 0.0];
    let mut initial_cursor_pos_set = false;
    let mut cursor_pos = [1.0, 1.0];

    const COLOR: [f32; 4] = [1.0, 1.0, 0.0, 0.1];

    let rect = rectangle::Rectangle::new(COLOR).border(rectangle::Border {
        color: COLOR,
        radius: 1.0,
    });
    while let Some(e) = events.next(&mut window) {
        e.mouse_cursor(|pos| {
            if initial_cursor_pos_set {
                let x: f64;
                let y: f64;
                if pos[0] > initial_cursor_pos[0] {
                    x = pos[0];
                } else {
                    x = initial_cursor_pos[0];
                };
                if pos[1] > initial_cursor_pos[1] {
                    y = pos[1];
                } else {
                    y = initial_cursor_pos[1];
                };
                cursor_pos = [x, y];
            } else {
                initial_cursor_pos = pos;
            };
        });
        if let Some(Button::Mouse(_)) = e.press_args() {
            initial_cursor_pos_set = true;
        };
        if let Some(Button::Mouse(_)) = e.release_args() {
            window.set_should_close(true);
        };
        if let Some(args) = e.render_args() {
            app.render(
                &args,
                &image,
                &texture,
                &rect,
                &initial_cursor_pos,
                &cursor_pos,
                &initial_cursor_pos_set,
                &draw_state,
            );
        }
    }
    return Some([
        initial_cursor_pos[0],
        initial_cursor_pos[1],
        cursor_pos[0] - initial_cursor_pos[0],
        cursor_pos[1] - initial_cursor_pos[1],
    ]);
}
