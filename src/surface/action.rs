// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Action, View};
#[cfg(feature = "winit")]
use crate::Application;

use iced::{Rectangle, window};
#[cfg(all(wayland_platform, feature = "winit"))]
use iced_runtime::platform_specific::wayland::CornerRadius;
#[cfg(wayland_platform)]
use iced_runtime::platform_specific::wayland::layer_surface::IcedMargin;
use std::any::Any;

use std::sync::Arc;

/// Used to produce a destroy popup message from within a widget.
#[cfg(any(wayland_platform, all(target_os = "macos", feature = "winit")))]
#[must_use]
pub fn destroy_popup<M>(id: iced_core::window::Id) -> Action<M> {
    Action::DestroyPopup(id)
}

#[cfg(wayland_platform)]
#[must_use]
pub fn destroy_subsurface<M>(id: iced_core::window::Id) -> Action<M> {
    Action::DestroySubsurface(id)
}

#[cfg(wayland_platform)]
#[must_use]
pub fn destroy_window<M>(id: iced_core::window::Id) -> Action<M> {
    Action::DestroyWindow(id)
}

#[cfg(wayland_platform)]
#[must_use]
pub fn destroy_layer_shell<M>(id: iced_core::window::Id) -> Action<M> {
    Action::DestroyLayerShell(id)
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LiveSettings {
    #[cfg(wayland_platform)]
    /// Override the surface padding value for the surface type.
    pub padding: Option<IcedMargin>,
    #[cfg(wayland_platform)]
    /// Override the default corner radius value for the surface type.
    pub corners: Option<CornerRadius>,
    /// Override the default blur setting for the surface type.
    pub blur: Option<bool>,
}

#[cfg(wayland_platform)]
type BoxedView<App> = Option<
    Box<
        dyn Fn(&App) -> crate::Element<'_, crate::Action<<App as Application>::Message>>
            + Send
            + Sync
            + 'static,
    >,
>;

#[cfg(wayland_platform)]
#[must_use]
pub fn app_window<App: Application>(
    live_settings: impl Fn(&App) -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn(&mut App) -> window::Settings + Send + Sync + 'static,
    view: BoxedView<App>,
) -> (window::Id, Action<App::Message>) {
    let id = window::Id::unique();

    let boxed: Box<dyn Fn(&mut App) -> window::Settings + Send + Sync + 'static> =
        Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn(&App) -> LiveSettings + Send + Sync + 'static> =
        Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    (
        id,
        Action::AppWindow(
            id,
            Arc::new(boxed),
            Arc::new(boxed_live),
            view.map(|view| {
                let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
                Arc::new(boxed)
            }),
        ),
    )
}

/// Used to create a window message from within a widget.
#[cfg(wayland_platform)]
#[must_use]
pub fn simple_window<Message: 'static>(
    live_settings: impl Fn() -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn() -> window::Settings + Send + Sync + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> (window::Id, Action<Message>) {
    let id = window::Id::unique();

    let boxed: Box<dyn Fn() -> window::Settings + Send + Sync + 'static> = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn() -> LiveSettings + Send + Sync + 'static> = Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    (
        id,
        Action::Window(
            id,
            Arc::new(boxed),
            Arc::new(boxed_live),
            view.map(|view| Arc::new(view) as View<Message>),
        ),
    )
}

#[cfg(wayland_platform)]
#[must_use]
pub fn app_popup<App: Application>(
    live_settings: impl Fn(&App) -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn(&mut App) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
    + Send
    + Sync
    + 'static,
    view: BoxedView<App>,
) -> Action<App::Message> {
    let boxed: Box<
        dyn Fn(&mut App) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn(&App) -> LiveSettings + Send + Sync + 'static> =
        Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    Action::AppPopup(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

/// Used to create a subsurface message from within a widget.
#[cfg(wayland_platform)]
#[must_use]
pub fn simple_subsurface<Message: 'static>(
    settings: impl Fn() -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        Box<dyn Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static>,
    >,
) -> Action<Message> {
    let boxed: Box<
        dyn Fn() -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    Action::Subsurface(
        Arc::new(boxed),
        Arc::new(Box::new(LiveSettings::default)),
        view.map(|view| Arc::from(view) as View<Message>),
    )
}

/// Where a menu popup goes on macOS and how big it is. See [`macos_popup`].
#[cfg(all(target_os = "macos", feature = "winit"))]
#[derive(Clone, Copy, Debug)]
pub struct MacosPopupSettings {
    /// The window the popup belongs to: the window a context menu opened in, or the popup
    /// a submenu opened from.
    pub parent: iced::window::Id,
    /// The popup's own window id.
    pub id: iced::window::Id,
    /// The popup's top-left corner, relative to the parent's content area. The popup is
    /// moved to stay on screen, as a Wayland compositor moves a reactive popup.
    pub offset: iced::Point,
    /// The popup's size.
    pub size: iced::Size,
    /// The x offset to use instead when the popup would cross the screen's right edge, so
    /// it flips to the other side of what opened it. `None` shifts it left instead.
    pub flip_x: Option<f32>,
}

/// Creates a menu popup from within a widget on macOS: a borderless panel window that can
/// extend past its parent window, standing in for a Wayland popup surface. Destroy it with
/// [`destroy_popup`], as on Wayland.
#[cfg(all(target_os = "macos", feature = "winit"))]
#[must_use]
pub fn macos_popup<Message: 'static>(
    settings: MacosPopupSettings,
    view: impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
) -> Action<Message> {
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(settings);
    let no_live_settings: Box<dyn Any + Send + Sync + 'static> = Box::new(());
    Action::Popup(
        Arc::new(boxed),
        Arc::new(no_live_settings),
        Some(Arc::new(view) as View<Message>),
    )
}

/// Used to create a popup message from within a widget.
#[cfg(wayland_platform)]
#[must_use]
pub fn simple_popup<Message: 'static>(
    live_settings: impl Fn() -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> Action<Message> {
    let boxed: Box<
        dyn Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn() -> LiveSettings + Send + Sync + 'static> = Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    Action::Popup(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| Arc::new(view) as View<Message>),
    )
}

#[cfg(wayland_platform)]
#[must_use]
pub fn subsurface<App: Application>(
    settings: impl Fn(
        &mut App,
    )
        -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
    + Send
    + Sync
    + 'static,
    // XXX Boxed trait object is required for less cumbersome type inference, but we box it anyways.
    view: BoxedView<App>,
) -> Action<App::Message> {
    let boxed: Box<
        dyn Fn(
                &mut App,
            )
                -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    Action::AppSubsurface(
        Arc::new(boxed),
        Arc::new(Box::new(|_: &App| LiveSettings::default())),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}

#[cfg(wayland_platform)]
#[must_use]
pub fn simple_layer_shell<Message: 'static>(
    live_settings: impl Fn() -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn()
        -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
    + Send
    + Sync
    + 'static,
    view: Option<
        impl Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync + 'static,
    >,
) -> Action<Message> {
    let boxed: Box<
        dyn Fn()
                -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);
    let boxed_live: Box<dyn Fn() -> LiveSettings + Send + Sync + 'static> = Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);
    Action::LayerShell(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| Arc::new(view) as View<Message>),
    )
}

#[cfg(wayland_platform)]
#[must_use]
pub fn app_layer_shell<App: Application>(
    live_settings: impl Fn(&App) -> LiveSettings + Send + Sync + 'static,
    settings: impl Fn(
        &mut App,
    )
        -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
    + Send
    + Sync
    + 'static,
    // XXX Boxed trait object is required for less cumbersome type inference, but we box it anyways.
    view: BoxedView<App>,
) -> Action<App::Message> {
    let boxed: Box<
        dyn Fn(
                &mut App,
            )
                -> iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings
            + Send
            + Sync
            + 'static,
    > = Box::new(settings);
    let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

    let boxed_live: Box<dyn Fn(&App) -> LiveSettings + Send + Sync + 'static> =
        Box::new(live_settings);
    let boxed_live: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed_live);

    Action::AppLayerShell(
        Arc::new(boxed),
        Arc::new(boxed_live),
        view.map(|view| {
            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(view);
            Arc::new(boxed)
        }),
    )
}
