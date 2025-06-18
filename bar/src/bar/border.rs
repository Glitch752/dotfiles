use std::{cell::RefCell, rc::Rc};

use cairo::{Context, FillRule};
use gtk4::prelude::*;
use palette::FromColor;

use crate::bar::{border::geom::{BorderState, Path, Rectangle}, BAR_THICKNESS, NON_BAR_BORDER_THICKNESS};

mod geom;

fn draw_path_with_rounded_corners(cr: &Context, mut path: Path, r: f64) {
    if path.is_empty() {
        return;
    }

    if path.len() < 3 {
        return;
    }
    path.unclose();
    let count = path.len();

    for i in 0..count+1 {
        let prev = path[(i - 1) % count];
        let cur = path[i % count];
        let next = path[(i + 1) % count];

        // The effective radius is the radius or half the distance to the next or previous point, whichever is smaller.
        let dist_prev = (prev - cur).len();
        let dist_next = (next - cur).len();
        let radius = r.min(dist_prev / 2.0).min(dist_next / 2.0);
        
        // Find the arc point
        let prev_norm = (prev - cur).unit();
        let next_norm = (next - cur).unit();

        if i == 0 {
            // For the first point, move to the start
            let start = path[0] + prev_norm * radius;
            cr.move_to(start.x, start.y);
        }

        // Don't arc for the last point, as it will be closed by the first point.
        if i == count {
            let end = path[0] + prev_norm * radius;
            cr.line_to(end.x, end.y);
            continue;
        }

        let arc_point = cur + (prev_norm + next_norm) * radius;

        let angle1 = next_norm.angle() + std::f64::consts::PI;
        let angle2 = prev_norm.angle() + std::f64::consts::PI;

        let outer_corner = prev_norm.cross(next_norm) < 0.0;
        if outer_corner {
            cr.arc(arc_point.x, arc_point.y, radius, angle1, angle2);
        } else {
            cr.arc_negative(arc_point.x, arc_point.y, radius, angle1, angle2);
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderWidget {
    canvas: Rc<gtk4::DrawingArea>,
    dynamic_cutin_state: Rc<RefCell<BorderState>>
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

        let state = BorderState::new();
        let state = Rc::new(RefCell::new(state));
        let state2 = state.clone();

        // TODO: We ideally wouldn't hardcode this in both SCSS and here, but for now we do.
        let background_color = palette::Srgb::new(17. / 255., 18. / 255., 27. / 255.); // #11121b
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

            let mut state = state2.borrow_mut();

            let (x1, y1) = (BAR_THICKNESS as f64, BAR_THICKNESS as f64);
            let (x2, y2) = (width as f64 - NON_BAR_BORDER_THICKNESS as f64, height as f64 - NON_BAR_BORDER_THICKNESS as f64);
            state.update_rectangles(
                Rectangle::filled_outward(x1, y1, x2, y2),
                Some(Rectangle::filled_inward_center(1500.0, 450.0, 200.0, 200.0))
            );

            // First, draw the borders
            cr.set_fill_rule(FillRule::EvenOdd);
            cr.set_source(&gradient).expect("Failed to set gradient source");

            let path = state.compute_border_path();

            cr.new_path();
            draw_path_with_rounded_corners(cr, path.clone(), corner_radius);
            cr.set_line_width(border_thickness * 2.);
            cr.stroke().expect("Failed to fill border path");

            // Now, draw the background
            cr.set_source_rgb(background_color.red as f64, background_color.green as f64, background_color.blue as f64);
            cr.new_path();
            // Fill the screen to invert the background
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            draw_path_with_rounded_corners(cr, path.clone(), corner_radius);
            cr.fill().expect("Failed to fill background path");
        });

        canvas.connect_resize(move |_, _width, _height| {
            // Redraw the border when the widget is resized
            canvas2.queue_draw();
        });

        Self { canvas, dynamic_cutin_state: state }
    }

    pub fn widget(self) -> Rc<gtk4::DrawingArea> {
        self.canvas
    }

    pub fn set_color(&self, color: &str) {
        self.canvas.set_css_classes(&[color]);
    }
}