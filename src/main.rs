extern crate sfml;

use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Color, Shape, Transformable};
use sfml::window::{ContextSettings, Event, Style, Key};
use sfml::system::{Vector2f};

fn main() {
  let mut window = create_window(640, 320);

  let mut sq = RectangleShape::new();
  sq.set_size(Vector2f::new(10.0, 10.0));
  sq.set_fill_color(&Color::rgb(255, 255, 255));
  sq.set_position((100., 100.));

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

// TODO: lifetime stuff - how to return struct with a lifetime

// fn create_pixel(x: u32, y: u32) -> sfml::graphics::RectangleShape {
//   let mut sq = RectangleShape::new();
//   sq.set_size(Vector2f::new(10.0, 10.0));
//   sq.set_fill_color(&Color::rgb(255, 255, 255));
//   sq.set_position((100., 100.));
//   sq
// }
