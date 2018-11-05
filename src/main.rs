extern crate sfml;
extern crate sfml_test;

use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Color, Shape, Transformable};
use sfml::window::{ContextSettings, Event, Style, Key};
use sfml::system::{Vector2f};

use sfml_test::Chip8;

fn main() {
  let mut window = create_window(640, 320);
  let sq = create_pixel(10., 10.);

  let cpu = Chip8::new();

  loop {
    while let Some(event) = window.poll_event() {
      match event {
        Event::Closed | Event::KeyPressed {
          code: Key::Escape, ..
        } => return,
        _ => {}
      }
    }

    window.clear(&Color::rgb(0, 0, 0));
    window.draw(&sq);
    window.display();
  }
}

fn create_window(width: u32, height: u32) -> sfml::graphics::RenderWindow {
  let context_settings = ContextSettings {
    antialiasing_level: 0,
    ..Default::default()
  };

  let mut window = RenderWindow::new(
    (width, height),
    "SFML Test",
    Style::CLOSE,
    &context_settings,
  );
  window.set_vertical_sync_enabled(true);
  window
}

fn create_pixel<'s>(x: f32, y: f32) -> sfml::graphics::RectangleShape<'s> {
  let mut sq = RectangleShape::new();
  sq.set_size(Vector2f::new(10.0, 10.0));
  sq.set_fill_color(&Color::rgb(255, 255, 255));
  sq.set_position((x, y));
  sq
}
