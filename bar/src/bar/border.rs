use std::{cell::RefCell, rc::Rc};

use cairo::{Context, FillRule};
use gtk4::prelude::*;
use palette::FromColor;

use crate::bar::{BAR_THICKNESS, NON_BAR_BORDER_THICKNESS};

#[derive(Debug, Clone)]
struct Rectangle {
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64
}

impl Rectangle {
    fn new(center_x: f64, center_y: f64, width: f64, height: f64) -> Self {
        Rectangle { center_x, center_y, width, height }
    }
}

fn inverse_rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64, screen_width: f64, screen_height: f64, inset: f64) {
    // Even-odd fill rule causes this to invert the rectangle
    cr.rectangle(0., 0., screen_width, screen_height);
    rounded_rect(cr, x, y, w, h, r, -inset);
}
fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64, inset: f64) {
    let r = r - inset; // Adjust radius by inset
    let (x, y, w, h) = (x + inset, y + inset, w - 2.0 * inset, h - 2.0 * inset);
    
    // top‑left corner
    cr.new_sub_path();
    cr.arc(x + r, y + r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
    // top edge → top‑right corner
    cr.arc(x + w - r, y + r, r, 3.0 * std::f64::consts::FRAC_PI_2, 0.0);
    // right edge → bottom‑right
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    // bottom edge → bottom‑left
    cr.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.close_path();
}

#[derive(Debug, Clone)]
pub struct BorderWidget {
    canvas: Rc<gtk4::DrawingArea>,
    bar: Rc<gtk4::ApplicationWindow>,
    cutin: Rc<RefCell<Option<Rectangle>>>
}

/// Interpolates two angles in degrees, ensuring the shortest path is taken.
fn interpolate_degrees_shorter(start: f32, end: f32, t: f32) -> f32 {
    let delta = (end - start).rem_euclid(360.0);
    let adjusted_end = if delta > 180.0 {
        end - 360.0
    } else if delta < -180.0 {
        end + 360.0
    } else {
        end
    };
    start + t * (adjusted_end - start)
}

trait LerpOklch {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}
impl LerpOklch for palette::Oklch {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let (l1, c1, h1) = (self.l, self.chroma, self.hue.into_positive_degrees());
        let (l2, c2, h2) = (other.l, other.chroma, other.hue.into_positive_degrees());
        let l = l1 + t * (l2 - l1);
        let c = c1 + t * (c2 - c1);
        let h = interpolate_degrees_shorter(h1, h2, t);
        palette::Oklch::new(l, c, h)
    }
}

impl BorderWidget {
    pub fn new(bar: Rc<gtk4::ApplicationWindow>) -> Self {
        let canvas = gtk4::DrawingArea::builder()
            .css_classes(["border-widget"])
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .build();

        let canvas = Rc::new(canvas);
        let canvas2 = canvas.clone();

        // let cutin = Rc::new(RefCell::new(None::<Rectangle>));
        let cutin = Rc::new(RefCell::new(Some(Rectangle::new(
            80.0, 200.0, 150.0, 150.0
        ))));
        let cutin2 = cutin.clone();

        let background_color = palette::Srgb::new(17. / 255., 18. / 255., 27. / 255.);
        let corner_radius = 16.0;
        let border_thickness = 2.0;

        // Since cairo doesn't support interpolation in oklch, we use palette to generate a few colors,
        // convert them to RGB, and use a linear gradient.

        // TODO: Sync this with niri configuration one way or the other
        let start_color: palette::Oklch = palette::Oklch::from_color(
            palette::Srgb::new(195. / 255., 55. / 255., 100. / 255.)
        ); // #c33764
        let end_color: palette::Oklch = palette::Oklch::from_color(
            palette::Srgb::new(29. / 255., 38. / 255., 113. / 255.)
        ); // #1d2671

        let stops = 20;
        let gradient_colors: Vec<(f64, f64, f64, f64)> = (0..stops)
            .map(|i| {
                let t = i as f64 / (stops as f64 - 1.); // Normalize to [0, 1]
                let color = start_color.lerp(&end_color, t as f32);
                let rgb = palette::Srgb::from_color(color);
                (t, rgb.red as f64, rgb.green as f64, rgb.blue as f64)
            })
            .collect();

        canvas.set_draw_func(move |_, cr, width, height| {
            let gradient = cairo::LinearGradient::new(0.0, height as f64, width as f64, 0.0);
            for(offset, r, g, b) in &gradient_colors {
                gradient.add_color_stop_rgba(*offset, *r, *g, *b, 1.0);
            }

            cr.set_source(&gradient).expect("Failed to set gradient source");

            let (x1, y1) = (BAR_THICKNESS as f64, BAR_THICKNESS as f64);
            let (x2, y2) =
                (width as f64 - NON_BAR_BORDER_THICKNESS as f64, height as f64 - NON_BAR_BORDER_THICKNESS as f64);

            cr.new_path();

            // First, draw the borders
            cr.set_fill_rule(FillRule::EvenOdd);
            inverse_rounded_rect(&cr, x1, y1, x2 - x1, y2 - y1, corner_radius, width as f64, height as f64, 0.);
            cr.fill().expect("Failed to fill rounded rectangle");
            if let Some(cutin) = cutin2.borrow().as_ref() {
                // Draw the cut-in rectangle
                rounded_rect(&cr, cutin.center_x - cutin.width / 2.0, cutin.center_y - cutin.height / 2.0,
                             cutin.width, cutin.height, corner_radius, 0.);
                cr.fill().expect("Failed to fill cut-in rectangle");
            }

            // Now, draw the background
            cr.set_source_rgb(background_color.red as f64, background_color.green as f64, background_color.blue as f64);
            inverse_rounded_rect(cr, x1, y1, x2 - x1, y2 - y1, corner_radius, width as f64, height as f64, border_thickness);
            cr.fill().expect("Failed to fill background rectangle");
            if let Some(cutin) = cutin2.borrow().as_ref() {
                // Draw the cut-in rectangle background
                cr.set_source_rgb(background_color.red as f64, background_color.green as f64, background_color.blue as f64);
                rounded_rect(cr, cutin.center_x - cutin.width / 2.0, cutin.center_y - cutin.height / 2.0,
                             cutin.width, cutin.height, corner_radius, border_thickness);
                cr.fill().expect("Failed to fill cut-in rectangle background");
            }

            println!("Drew border with width: {}, height: {}", width, height);
        });

        canvas.connect_resize(move |_, width, height| {
            // Redraw the border when the widget is resized
            canvas2.queue_draw();
        });

        Self {
            canvas,
            bar,
            cutin
        }
    }

    pub fn widget(self) -> Rc<gtk4::DrawingArea> {
        self.canvas
    }

    pub fn set_color(&self, color: &str) {
        self.canvas.set_css_classes(&[color]);
    }
}