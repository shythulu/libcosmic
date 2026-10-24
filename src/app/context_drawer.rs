// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0
//
use std::borrow::Cow;

use crate::Element;

pub struct ContextDrawer<'a, Message: Clone + 'static> {
    pub title: Option<Cow<'a, str>>,
    pub actions: Option<Element<'a, Message>>,
    pub header: Option<Element<'a, Message>>,
    pub content: Element<'a, Message>,
    pub footer: Option<Element<'a, Message>>,
    pub on_close: Message,
    /// The width the app asks for. The drawer still leaves room for the content.
    pub width: Option<f32>,
    /// Called with the new width while the user drags the drawer's left edge.
    pub on_resize: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    /// Emitted when the drag on the drawer's edge ends.
    pub on_resize_end: Option<Message>,
}

#[cfg(feature = "about")]
pub fn about<'a, Message: Clone + 'static>(
    about: &'a crate::widget::about::About,
    on_url_press: impl Fn(&'a str) -> Message + 'a,
    on_close: Message,
) -> ContextDrawer<'a, Message> {
    context_drawer(crate::widget::about(about, on_url_press), on_close)
}

pub fn context_drawer<'a, Message: Clone + 'static>(
    content: impl Into<Element<'a, Message>>,
    on_close: Message,
) -> ContextDrawer<'a, Message> {
    ContextDrawer {
        title: None,
        actions: None,
        header: None,
        content: content.into(),
        footer: None,
        on_close,
        width: None,
        on_resize: None,
        on_resize_end: None,
    }
}

impl<'a, Message: Clone + 'static> ContextDrawer<'a, Message> {
    /// Set a context drawer title
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// App-specific actions at the top-left corner of the context drawer
    pub fn actions(mut self, actions: impl Into<Element<'a, Message>>) -> Self {
        self.actions = Some(actions.into());
        self
    }

    /// Elements placed above the context drawer scrollable
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Elements placed below the context drawer scrollable
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Ask for a width, such as one the user chose earlier by dragging the edge.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Let the user resize the drawer by dragging its left edge. `on_resize` gets the
    /// width the drawer would have; pass it back through [`Self::width`].
    pub fn on_resize(mut self, on_resize: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(on_resize));
        self
    }

    /// Emitted once when a resize drag ends, for saving the width.
    pub fn on_resize_end(mut self, message: Message) -> Self {
        self.on_resize_end = Some(message);
        self
    }

    pub fn map<Out: Clone + 'static>(
        self,
        on_message: fn(Message) -> Out,
    ) -> ContextDrawer<'a, Out> {
        ContextDrawer {
            title: self.title,
            actions: self.actions.map(|el| el.map(on_message)),
            header: self.header.map(|el| el.map(on_message)),
            content: self.content.map(on_message),
            footer: self.footer.map(|el| el.map(on_message)),
            on_close: on_message(self.on_close),
            width: self.width,
            on_resize: self.on_resize.map(|on_resize| {
                Box::new(move |width| on_message(on_resize(width))) as Box<dyn Fn(f32) -> Out + 'a>
            }),
            on_resize_end: self.on_resize_end.map(on_message),
        }
    }
}
