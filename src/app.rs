// SPDX-License-Identifier: GPL-3.0

use cosmic::Element;
use cosmic::iced::Alignment::Center;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::widget::{column, row};
use cosmic::iced::{Limits, Subscription, window::Id};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::widget::{TextInput, text};
use std::time::Duration;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Example row toggler.
    intervals: u32,
    current_interval: u32,
    timer_value: u32,
    break_value: u32,
    break_count: u32,
    current_break: u32,
    on_break: bool,
    remaining: u32,
    running: bool,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Increment,
    Decrement,
    IncrementBreak,
    DecrementBreak,
    IncrementInterval,
    DecrementInterval,
    StartTimer,
    StopTimer,
    Tick,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.pojomi.pomodoro";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            timer_value: 25,
            break_value: 5,
            remaining: 25,
            running: false,
            intervals: 4,
            current_interval: 0,
            break_count: 3,
            current_break: 0,
            on_break: false,
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("alarm-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let content = column![
            row![
                widget::icon::from_name("value-decrease-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::Decrement),
                text(format!("{:02}:00", self.timer_value)),
                widget::icon::from_name("value-increase-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::Increment),
            ]
            .spacing(8)
            .align_y(Center),
            row![
                widget::icon::from_name("value-decrease-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::DecrementBreak),
                text(format!("{:02}:00", self.break_value)),
                widget::icon::from_name("value-increase-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::IncrementBreak)
            ]
            .spacing(8)
            .align_y(Center),
            row![
                widget::icon::from_name("value-decrease-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::DecrementInterval),
                text(format!("Interval: {}", self.intervals)),
                widget::icon::from_name("value-increase-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::IncrementInterval)
            ]
            .spacing(8)
            .align_y(Center),
            row![
                widget::icon::from_name("media-playback-stop-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::StopTimer),
                widget::icon::from_name("media-playback-start-symbolic")
                    .size(16)
                    .apply(widget::button::icon)
                    .on_press(Message::StartTimer),
            ]
            .spacing(8)
            .align_y(Center),
            text(format!(
                "{:02}:{:02}",
                self.remaining / 60,
                self.remaining % 60
            )),
            text(format!("{} of {}", self.current_interval, self.intervals))
        ]
        .spacing(8)
        .padding(8)
        .align_x(Center);
        self.core.applet.popup_container(content).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.running {
            cosmic::iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Increment => self.timer_value += 1,
            Message::Decrement => {
                if self.timer_value > 0 {
                    self.timer_value -= 1;
                }
            }
            Message::IncrementBreak => self.break_value += 1,
            Message::DecrementBreak => {
                if self.break_value > 0 {
                    self.break_value -= 1;
                }
            }
            Message::IncrementInterval => {
                self.intervals += 1;
                self.break_count += 1;
            }
            Message::DecrementInterval => {
                if self.intervals > 1 {
                    self.intervals -= 1;
                    self.break_count -= 1;
                }
            }
            Message::StopTimer => {
                if self.running {
                    self.running = false;
                    notify_done(false);
                }
            }
            Message::StartTimer => {
                self.remaining = self.timer_value * 60;
                self.current_interval += 1;
                self.running = true;
            }
            Message::Tick => {
                self.remaining -= 1;
                if self.remaining == 0 {
                    if self.current_interval == self.intervals {
                        self.running = false;
                        notify_done(true);
                    } else {
                        if !self.on_break {
                            self.on_break = true;
                            self.current_break += 1;
                            self.remaining = self.break_value * 60;
                            notify_break(self.current_break, self.break_count);
                        } else {
                            self.on_break = false;
                            self.remaining = self.timer_value * 60;
                            self.current_interval += 1;
                            notify_next(self.current_interval, self.intervals);
                        }
                    }
                }
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
fn notify_break(current_break: u32, breaks: u32) {
    let _ = notify_rust::Notification::new()
        .summary("Begin break")
        .body(format!("Break {} of {}", current_break, breaks).as_str())
        .icon("alarm-symbolic")
        .show();
}

fn notify_next(current_interval: u32, total_intervals: u32) {
    let _ = notify_rust::Notification::new()
        .summary("Begin next interval")
        .body(format!("Interval {} of {}", current_interval, total_intervals).as_str())
        .icon("alarm-symbolic")
        .show();
}

fn notify_done(completed: bool) {
    if completed {
        let _ = notify_rust::Notification::new()
            .summary("Timer finished")
            .body("Time's up!")
            .icon("alarm-symbolic")
            .show();
    } else {
        let _ = notify_rust::Notification::new()
            .summary("Timer stopped")
            .body("The timer was stopped")
            .icon("alarm-symbolic")
            .show();
    }
}
