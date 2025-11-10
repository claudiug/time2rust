use chrono::Utc;
use gpui::{
    App, Application, Bounds, Context, Entity, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::{ActiveTheme as _, Sizable, tag::Tag};

#[derive(Debug, Clone)]
pub struct WorldTime {
    name: String,
    time: String,        // HH:MM format
    diff_hours: i32,     // hours difference from home time
    is_home: bool,       // true if this is your home location
    timezone_id: String, // like "Europe/Berlin" or "America/Chicago"
}

impl WorldTime {
    fn new(name: &str, timezone_id: &str, is_home: bool, home_offset: i32) -> Self {
        WorldTime {
            name: name.to_string(),
            time: Self::calculate_time_from_austin(home_offset),
            diff_hours: home_offset,
            is_home,
            timezone_id: timezone_id.to_string(),
        }
    }

    // Calculate Austin's time (UTC-6) as the base
    fn get_austin_time() -> chrono::DateTime<Utc> {
        Utc::now() + chrono::Duration::hours(-6) // Austin is UTC-6
    }

    // Calculate time relative to Austin's time
    fn calculate_time_from_austin(austin_offset: i32) -> String {
        let austin_time = Self::get_austin_time();
        let adjusted_time = austin_time + chrono::Duration::hours(austin_offset as i64);
        adjusted_time.format("%H:%M").to_string()
    }

    fn update_time(&mut self) {
        self.time = Self::calculate_time_from_austin(self.diff_hours);
    }
}

impl Render for WorldTime {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let frame_color = if self.is_home {
            rgb(0x3b82f6) // Blue border for home
        } else {
            rgb(0x6b7280) // Gray for others
        };

        let bg_color = if self.is_home {
            rgb(0xf0f9ff) // Light blue background for home
        } else {
            rgb(0xf9fafb) // Light gray for others
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .min_w(px(180.0))
            .bg(bg_color)
            .border_2()
            .border_color(frame_color)
            .rounded(px(8.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .child(self.name.to_string())
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(if self.is_home {
                                        rgb(0x3b82f6)
                                    } else {
                                        rgb(0x111827)
                                    }),
                            )
                            .children(self.is_home.then(|| Tag::secondary().small().child("Home"))),
                    )
                    .child(
                        div().flex().items_center().gap_2().child(
                            div()
                                .child(self.time.to_string())
                                .text_3xl()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(0x111827)),
                        ),
                    )
                    .child(
                        div()
                            .child(format!("Δ {} hours", self.diff_hours).to_string())
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(if self.diff_hours >= 0 {
                                rgb(0x22c55e)
                            } else {
                                rgb(0xef4444)
                            }),
                    )
                    .child(
                        div().flex().items_center().gap_1().child(
                            div()
                                .child(self.timezone_id.to_string())
                                .text_xs()
                                .text_color(rgb(0x6b7280)),
                        ),
                    ),
            )
    }
}
struct WorldTimeApp {
    cities: Vec<Entity<WorldTime>>,
    last_update: std::time::Instant,
}

impl Render for WorldTimeApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Update times every minute
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update).as_secs() >= 60 {
            for city in &self.cities {
                city.update(cx, |city, _cx| {
                    city.update_time();
                });
            }
            self.last_update = now;
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(cx.theme().background)
            .size_full()
            .child(
                div()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child("🌍 World Time Display"),
                    )
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(cx.theme().accent_foreground)
                    .text_center(),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_8()
                    .justify_center()
                    .children(self.cities.iter().map(|city| city.clone())),
            )
    }
}
fn main() {
    Application::new().run(|cx: &mut App| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        // Handle window closing - quit app when last window closes
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("🌍 World Time Display")),
                    ..Default::default()
                }),
                show: true,
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    // Prevent window maximization
                    cx.observe_window_bounds(window, move |_, window, _cx| {
                        if window.is_maximized() {
                            // Restore to original size when maximized
                            window.resize(size(px(800.0), px(600.0)));
                        }
                    })
                    .detach();

                    let austin = cx.new(|_| WorldTime::new("Austin", "America/Chicago", true, 0));
                    let nyc = cx.new(|_| WorldTime::new("NYC", "America/New_York", false, 1));
                    let london = cx.new(|_| WorldTime::new("London", "Europe/London", false, 6));
                    let berlin = cx.new(|_| WorldTime::new("Berlin", "Europe/Berlin", false, 7));
                    let bucharest =
                        cx.new(|_| WorldTime::new("Bucharest", "Europe/Bucharest", false, 8));

                    WorldTimeApp {
                        cities: vec![austin, nyc, london, berlin, bucharest],
                        last_update: std::time::Instant::now(),
                    }
                })
            },
        )
        .unwrap();
    });
}
