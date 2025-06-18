use std::{any::TypeId, rc::Rc};

use gtk4::prelude::*;

use crate::{bar::{BAR_THICKNESS, border::geom::Rectangle}, App};

pub mod launcher;

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
}

#[derive(Debug)]
pub struct Popouts {
    pub app: Option<Rc<App>>,
    pub open: Vec<OpenPopout>,
    pub container: Option<gtk4::Fixed>
}

impl Popouts {
    pub fn new() -> Self {
        Popouts {
            app: None,
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

    pub fn open_popout<T: 'static>(&mut self, id: &str, widget: gtk4::Widget, takes_keyboard_focus: bool) {
        // If the popout is already open, close it
        let module_id = TypeId::of::<T>();
        if let Some((i, popout)) = self.open.iter().enumerate().find(
            |(_, popout)| popout.module_id == module_id
        ) {
            if let Some(container) = &self.container {
                container.remove(&popout.widget);
            }
            self.open.remove(i);
            return;
        }

        let target_size = widget.size_request();
        let initial_position = (BAR_THICKNESS as f64, BAR_THICKNESS as f64);
        self.open.push(
            OpenPopout {
                module_id,
                interpolation_id: id.to_string(),
                widget,
                position: initial_position,
                size: (target_size.0 as f64, target_size.1 as f64),
                
                target_size: (target_size.0 as f64, target_size.1 as f64),
                target_position: (0.0, 0.0),
                
                anchor: AnchorPoint::TopLeft,
                takes_keyboard_focus
            }
        );

        if let Some(container) = &self.container {
            container.put(&self.open.last().unwrap().widget, initial_position.0, initial_position.1);
            self.app.clone().unwrap().queue_begin_animation();
            self.open.last().unwrap().widget.show();
        }
    }
}