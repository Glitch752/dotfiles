use std::{any::TypeId, time::Duration};

use gtk4::prelude::*;

use crate::{bar::{border::geom::Rectangle, BAR_THICKNESS}, modules::Module};

#[derive(Debug)]
#[allow(unused)]
pub enum AnchorPoint {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug)]
pub struct OpenPopout {
    pub module_id: TypeId,
    pub interpolation_id: String,

    pub widget: gtk4::Widget,
    pub position: (f64, f64),
    pub size: (f64, f64),

    pub target_position: (f64, f64),
    pub target_size: (f64, f64),

    pub anchor: AnchorPoint,
    pub takes_keyboard_focus: bool
}

fn exponential_smoothing(mut value: f64, target_value: f64, delta: Duration, speed: f64) -> f64 {
    value += (target_value - value) * (1. - (delta.as_secs_f64() * -speed).exp());
    if (value - target_value).abs() < 1e-5 {
        return target_value;
    }
    return value;
}

impl OpenPopout {
    pub fn get_rectangle(&self) -> Rectangle {
        let position = match self.anchor {
            AnchorPoint::TopLeft => self.position,
            AnchorPoint::TopRight => (self.position.0 - self.size.0, self.position.1),
            AnchorPoint::BottomLeft => (self.position.0, self.position.1 - self.size.1),
            AnchorPoint::BottomRight => (self.position.0 - self.size.0, self.position.1 - self.size.1)
        };
        let size = self.size;

        Rectangle::filled_inward(
            position.0, position.1,
            position.0 + size.0, position.1 + size.1
        )
    }

    /// Animates this popout. Returns if the animation should continue (e.g. things are still moving)
    pub fn animate(&mut self, dt: Duration) -> bool {
        let start_position = self.position;
        let start_size = self.size;

        let speed = 20.;

        self.position = (
            exponential_smoothing(self.position.0, self.target_position.0, dt, speed),
            exponential_smoothing(self.position.1, self.target_position.1, dt, speed)
        );
        self.size = (
            exponential_smoothing(self.size.0, self.target_size.0, dt, speed),
            exponential_smoothing(self.size.1, self.target_size.1, dt, speed)
        );

        return self.position != start_position || self.size != start_size || dt.as_millis() < 1;
    }
}

#[derive(Debug)]
pub struct Popouts {
    pub open: Vec<OpenPopout>,
    pub container: Option<gtk4::Fixed>
}

impl Popouts {
    pub fn new() -> Self {
        Popouts {
            open: Vec::new(),
            container: None
        }
    }

    pub fn init_container(&mut self) -> gtk4::Fixed {
        let container = gtk4::Fixed::new();
        container.add_css_class("popout-container");
        self.container = Some(container.clone());
        container
    }

    pub fn open_popout<SourceModule: 'static + Module>(&mut self, id: &str, widget: gtk4::Widget, takes_keyboard_focus: bool) {
        // If the popout is already open, close it
        let module_id = TypeId::of::<SourceModule>();
        if let Some((i, popout)) = self.open.iter().enumerate().find(
            |(_, popout)| popout.module_id == module_id
        ) {
            if let Some(container) = &self.container {
                container.remove(&popout.widget);
            }
            self.open.remove(i);
            return;
        }

        let initial_size = widget.size_request();
        let initial_position = (BAR_THICKNESS as f64 - 5.0, BAR_THICKNESS as f64 - 5.0);
        self.open.push(OpenPopout {
            module_id,
            interpolation_id: id.to_string(),
            widget,
            position: initial_position,
            size: (initial_size.0 as f64, 0.0),
            
            target_size: (initial_size.0 as f64, initial_size.1 as f64),
            target_position: initial_position,
            
            anchor: AnchorPoint::TopLeft,
            takes_keyboard_focus
        });

        if let Some(container) = &self.container {
            container.put(&self.open.last().unwrap().widget, initial_position.0, initial_position.1);
            self.open.last().unwrap().widget.show();
        }
    }

    /// Returns if we should continue animating.
    pub fn animate(&mut self, dt: Duration) -> bool {
        let continue_animating = self.open.iter_mut().any(|open| open.animate(dt));

        if let Some(container) = &self.container {
            for OpenPopout { widget, position, .. } in &self.open {
                // container.move_(widget, position.0, position.1);
                // TODO: This breaks gtk??
            }
        }

        return continue_animating;
    }
}