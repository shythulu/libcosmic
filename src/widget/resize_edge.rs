// Copyright 2026 Macosmetic contributors
// SPDX-License-Identifier: MPL-2.0

//! A drag handle on one vertical edge of a widget, for panels the user can resize.
//!
//! The widget lays out its content unchanged. Pressing near the chosen edge starts a drag,
//! and every cursor move reports the width the content would have if the edge followed the
//! cursor. The app stores that width and lays the content out at it on the next view.

use iced::{Event, Length, Rectangle, Size, Vector, mouse};
use iced_core::widget::{Operation, Tree, tree};
use iced_core::{Border, Clipboard, Layout, Shell, Widget, layout, overlay, renderer};

use crate::Element;

/// How far the grab strip reaches into the content, in logical pixels.
const INSIDE: f32 = 3.0;
/// How far the grab strip reaches past the content, into the gap beside it.
const OUTSIDE: f32 = 6.0;
/// Width of the line drawn on the edge while it is hovered or dragged.
const LINE: f32 = 2.0;

/// The edge of the content that the user drags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The left edge. Dragging left makes the content wider.
    Left,
    /// The right edge. Dragging right makes the content wider.
    Right,
}

/// Wraps `content` with a drag handle on `edge`.
pub fn resize_edge<'a, Message: Clone + 'static>(
    content: impl Into<Element<'a, Message>>,
    edge: Edge,
    on_resize: impl Fn(f32) -> Message + 'a,
) -> ResizeEdge<'a, Message> {
    ResizeEdge {
        content: content.into(),
        edge,
        on_resize: Box::new(on_resize),
        on_release: None,
    }
}

pub struct ResizeEdge<'a, Message> {
    content: Element<'a, Message>,
    edge: Edge,
    on_resize: Box<dyn Fn(f32) -> Message + 'a>,
    on_release: Option<Message>,
}

impl<Message> ResizeEdge<'_, Message> {
    /// Emitted once when a drag ends, for saving the final width.
    #[must_use]
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    fn strip(&self, bounds: Rectangle) -> Rectangle {
        let x = match self.edge {
            Edge::Left => bounds.x - OUTSIDE,
            Edge::Right => bounds.x + bounds.width - INSIDE,
        };
        Rectangle {
            x,
            y: bounds.y,
            width: INSIDE + OUTSIDE,
            height: bounds.height,
        }
    }

    fn line(&self, bounds: Rectangle) -> Rectangle {
        let x = match self.edge {
            Edge::Left => bounds.x - OUTSIDE / 2.0 - LINE / 2.0,
            Edge::Right => bounds.x + bounds.width + OUTSIDE / 2.0 - LINE / 2.0,
        };
        Rectangle {
            x,
            y: bounds.y,
            width: LINE,
            height: bounds.height,
        }
    }
}

#[derive(Default)]
struct State {
    /// During a drag, the x of the content's other edge, which stays put. The width is
    /// the distance from it to the cursor, so a stale press position cannot skew it.
    drag: Option<f32>,
}

impl<Message: Clone> Widget<Message, crate::Theme, crate::Renderer> for ResizeEdge<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();

        match event {
            // The strip goes first, so a scrollbar inside the content's edge cannot take
            // the press.
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if !shell.is_event_captured() && cursor.is_over(self.strip(bounds)) =>
            {
                state.drag = Some(match self.edge {
                    Edge::Left => bounds.x + bounds.width,
                    Edge::Right => bounds.x,
                });
                shell.capture_event();
                return;
            }
            Event::Mouse(mouse::Event::CursorMoved { position, .. }) => {
                if let Some(anchor) = state.drag {
                    let width = match self.edge {
                        Edge::Left => anchor - position.x,
                        Edge::Right => position.x - anchor,
                    };
                    shell.publish((self.on_resize)(width.max(0.0)));
                    shell.capture_event();
                    return;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag.is_some() =>
            {
                state.drag = None;
                if let Some(message) = self.on_release.clone() {
                    shell.publish(message);
                }
                shell.capture_event();
                return;
            }
            _ => {}
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if state.drag.is_some() || cursor.is_over(self.strip(layout.bounds())) {
            return mouse::Interaction::ResizingHorizontally;
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        if state.drag.is_some() || cursor.is_over(self.strip(bounds)) {
            let cosmic = theme.cosmic();
            iced_core::Renderer::fill_quad(
                renderer,
                renderer::Quad {
                    bounds: self.line(bounds),
                    border: Border {
                        radius: (LINE / 2.0).into(),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Color::from(cosmic.accent_color()),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &crate::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.content.as_widget().drag_destinations(
            &tree.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }

    #[cfg(feature = "a11y")]
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        tree: &Tree,
        cursor: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        self.content
            .as_widget()
            .a11y_nodes(layout, &tree.children[0], cursor)
    }
}

impl<'a, Message: Clone + 'static> From<ResizeEdge<'a, Message>> for Element<'a, Message> {
    fn from(widget: ResizeEdge<'a, Message>) -> Self {
        Element::new(widget)
    }
}
