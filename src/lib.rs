//! `drawille` – a terminal graphics library for Rust, based on the Python library
//! [drawille](https://github.com/asciimoo/drawille).
//!
//! This crate provides an interface for utilising Braille characters to draw a picture to a
//! terminal, allowing for much smaller pixels but losing proper colour support.
//!
//! # Example
//!
//! ```
//! extern crate drawille;
//!
//! use drawille::Canvas;
//!
//! fn main() {
//!     let mut canvas = Canvas::new(10, 10);
//!     canvas.set(5, 4);
//!     canvas.line(2, 2, 8, 8);
//!     assert_eq!(canvas.frame(), [
//! " ⢄    ",
//! "  ⠙⢄  ",
//! "    ⠁ "].join("\n"));
//! }
//! ```

use std::cmp;
use std::f32;

extern crate fnv;
use fnv::FnvHashMap;

mod pixel;

use pixel::{colorize_char, Color, Pixel};

static PIXEL_MAP: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

/// A canvas object that can be used to draw to the terminal using Braille characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    pixels: FnvHashMap<(u16, u16), Pixel>,
    width: u16,
    height: u16,
    color: Option<Color>,
}

impl Canvas {
    /// Creates a new `Canvas` with the given width and height.
    ///
    /// Note that the `Canvas` can still draw outside the given dimensions (expanding the canvas)
    /// if a pixel is set outside the dimensions.
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            pixels: FnvHashMap::default(),
            width: (width / 2) as u16,
            height: (height / 4) as u16,
            color: None,
        }
    }

    /// Clears the canvas.
    pub fn clear(&mut self) {
        self.pixels.clear();
    }

    /// Sets default color on drawing
    pub fn set_color(&mut self, color: u32) {
        self.color = Some(Color::from_hex(color));
    }

    /// Resets default color
    pub fn reset_color(&mut self) {
        self.color = None
    }

    /// Sets a pixel at the specified coordinates.
    pub fn set(&mut self, x: u32, y: u32) {
        let (row, col) = ((x / 2) as u16, (y / 4) as u16);
        let pixel = self.pixels.entry((row, col)).or_default();
        pixel.set_line(PIXEL_MAP[y as usize % 4][x as usize % 2]);
        pixel.add_color(self.color);
    }

    /// Sets a letter at the specified coordinates.
    pub fn set_char(&mut self, x: u32, y: u32, c: char) {
        let (row, col) = ((x / 2) as u16, (y / 4) as u16);
        let pixel = self.pixels.entry((row, col)).or_default();
        pixel.set_char(c);
        pixel.set_color(self.color);
    }

    /// Deletes a letter at the speified coordinates
    pub fn unset_char(&mut self, x: u32, y: u32) {
        let (row, col) = ((x / 2) as u16, (y / 4) as u16);
        self.pixels
            .entry((row, col))
            .and_modify(|pixel| pixel.unset_char());
    }

    /// Draws text at the specified coordinates (top-left of the text) up to max_width length
    pub fn text(&mut self, x: u32, y: u32, max_width: u32, text: &str) {
        for (i, c) in text.chars().enumerate() {
            let w = i as u32 * 2;
            if w > max_width {
                return;
            }
            self.set_char(x + w, y, c);
        }
    }

    /// Deletes a pixel at the specified coordinates.
    pub fn unset(&mut self, x: u32, y: u32) {
        let (row, col) = ((x / 2) as u16, (y / 4) as u16);
        self.pixels
            .entry((row, col))
            .and_modify(|pixel| pixel.unset_line(PIXEL_MAP[y as usize % 4][x as usize % 2]));
    }

    /// Toggles a pixel at the specified coordinates.
    pub fn toggle(&mut self, x: u32, y: u32) {
        let (row, col) = ((x / 2) as u16, (y / 4) as u16);
        self.pixels
            .entry((row, col))
            .and_modify(|pixel| pixel.toggle_line(PIXEL_MAP[y as usize % 4][x as usize % 2]));
    }

    /// Detects whether the pixel at the given coordinates is set.
    pub fn get(&self, x: u32, y: u32) -> bool {
        let (row, col) = ((x / 2) as u16, (y / 4) as u16);
        self.pixels.get(&(row, col)).map_or(false, |pixel| {
            pixel.get_line(PIXEL_MAP[y as usize % 4][x as usize % 2])
        })
    }

    /// Returns a `Vec` of each row of the `Canvas`.
    ///
    /// Note that each row is actually four pixels high due to the fact that a single Braille
    /// character spans two by four pixels.
    pub fn rows(&self) -> Vec<String> {
        let mut maxrow = self.width;
        let mut maxcol = self.height;
        for &(x, y) in self.pixels.keys() {
            if x > maxrow {
                maxrow = x;
            }
            if y > maxcol {
                maxcol = y;
            }
        }

        let mut result = Vec::with_capacity(maxcol as usize + 1);
        for y in 0..=maxcol {
            let mut row = String::with_capacity(maxrow as usize + 1);
            let mut prev_color: Option<Color> = None;
            for x in 0..=maxrow {
                let cell = self.pixels.get(&(x, y)).cloned().unwrap_or_default();
                let (s, color) = cell.get_char(prev_color);
                row.extend(s.chars());
                prev_color = color;
            }
            if prev_color.is_some() {
                row.extend(colorize_char(None, None, true).chars());
            }
            result.push(row);
        }
        result
    }

    /// Draws the canvas to a `String` and returns it.
    pub fn frame(&self) -> String {
        self.rows().join("\n")
    }

    /// Draws a line from `(x1, y1)` to `(x2, y2)` onto the `Canvas`.
    pub fn line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        let xdiff = cmp::max(x1, x2) - cmp::min(x1, x2);
        let ydiff = cmp::max(y1, y2) - cmp::min(y1, y2);
        let xdir = if x1 <= x2 { 1 } else { -1 };
        let ydir = if y1 <= y2 { 1 } else { -1 };

        let r = cmp::max(xdiff, ydiff);

        for i in 0..=r {
            let mut x = x1 as i32;
            let mut y = y1 as i32;

            if ydiff != 0 {
                y += ((i * ydiff) / r) as i32 * ydir;
            }
            if xdiff != 0 {
                x += ((i * xdiff) / r) as i32 * xdir;
            }

            self.set(x as u32, y as u32);
        }
    }
}

/// A ‘turtle’ that can walk around a canvas drawing lines.
pub struct Turtle {
    pub x: f32,
    pub y: f32,
    pub brush: bool,
    pub rotation: f32,
    pub cvs: Canvas,
}

impl Turtle {
    /// Create a new `Turtle`, starting at the given coordinates.
    ///
    /// The turtle starts with its brush down, facing right.
    pub fn new(x: f32, y: f32) -> Turtle {
        Turtle {
            cvs: Canvas::new(0, 0),
            x: x,
            y: y,
            brush: true,
            rotation: 0.0,
        }
    }

    /// Creates a new `Turtle` with the provided `Canvas`, starting at the given coordinates.
    ///
    /// The turtle starts with its brush down, facing right.
    pub fn from_canvas(x: f32, y: f32, cvs: Canvas) -> Turtle {
        Turtle {
            cvs: cvs,
            x: x,
            y: y,
            brush: true,
            rotation: 0.0,
        }
    }

    /// Sets the width of a `Turtle`’s `Canvas`, and return it for use again.
    pub fn width(mut self, width: u32) -> Turtle {
        self.cvs.width = width as u16;
        self
    }

    /// Sets the height of a `Turtle`’s `Canvas`, and return it for use again.
    pub fn height(mut self, height: u32) -> Turtle {
        self.cvs.height = height as u16;
        self
    }

    /// Lifts the `Turtle`’s brush.
    pub fn up(&mut self) {
        self.brush = false;
    }

    /// Puts down the `Turtle`’s brush.
    pub fn down(&mut self) {
        self.brush = true;
    }

    /// Toggles the `Turtle`’s brush.
    pub fn toggle(&mut self) {
        self.brush = !self.brush;
    }

    /// Moves the `Turtle` forward by `dist` steps.
    pub fn forward(&mut self, dist: f32) {
        let x = self.x + degrees_to_radians(self.rotation).cos() * dist;
        let y = self.y + degrees_to_radians(self.rotation).sin() * dist;
        self.teleport(x, y);
    }

    /// Moves the `Turtle` backward by `dist` steps.
    pub fn back(&mut self, dist: f32) {
        self.forward(-dist);
    }

    /// Teleports the `Turtle` to the given coordinates.
    ///
    /// Note that this draws a line between the old position and the new one if the `Turtle`’s
    /// brush is down.
    pub fn teleport(&mut self, x: f32, y: f32) {
        if self.brush {
            self.cvs.line(
                cmp::max(0, self.x.round() as i32) as u32,
                cmp::max(0, self.y.round() as i32) as u32,
                cmp::max(0, x.round() as i32) as u32,
                cmp::max(0, y.round() as i32) as u32,
            );
        }

        self.x = x;
        self.y = y;
    }

    /// Turns the `Turtle` right (clockwise) by `angle` degrees.
    pub fn right(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// Turns the `Turtle` left (clockwise) by `angle` degrees.
    pub fn left(&mut self, angle: f32) {
        self.rotation -= angle;
    }

    /// Writes the `Turtle`’s `Canvas` to a `String` and returns it.
    pub fn frame(&self) -> String {
        self.cvs.frame()
    }
}

fn degrees_to_radians(deg: f32) -> f32 {
    deg * (f32::consts::PI / 180.0f32)
}
