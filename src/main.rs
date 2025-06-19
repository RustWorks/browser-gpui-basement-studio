use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use gpui::{
    div, linear_color_stop, linear_gradient, point, prelude::*, px, rgb, rgba, size, svg, App,
    AppContext, Application, AssetSource, Bounds, Context, Entity, IntoElement, ParentElement,
    Render, SharedString, Styled, Timer, Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    input::{InputEvent, InputState, TextInput},
    Root,
};
use gpui_webview::{
    events::TitleChangedEvent,
    wef::{self, Frame, FuncRegistry, Settings},
    WebView,
};
use serde::Serialize;

// Asset loader for SVG files
struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let full_path = self.base.join(path);

        match fs::read(&full_path) {
            Ok(data) => Ok(Some(std::borrow::Cow::Owned(data))),
            Err(err) => {
                println!("Failed to load asset: {:?} - Error: {}", full_path, err);
                Err(err.into())
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let full_path = self.base.join(path);

        match fs::read_dir(&full_path) {
            Ok(entries) => {
                let files: Vec<SharedString> = entries
                    .filter_map(|entry| match entry {
                        Ok(entry) => {
                            let file_name = entry.file_name();
                            file_name.into_string().ok().map(SharedString::from)
                        }
                        Err(err) => {
                            println!("Error reading directory entry: {}", err);
                            None
                        }
                    })
                    .collect();

                println!("Listed {} files in directory: {:?}", files.len(), full_path);
                Ok(files)
            }
            Err(err) => {
                println!("Failed to list directory: {:?} - Error: {}", full_path, err);
                Err(err.into())
            }
        }
    }
}

// SVG button component
fn svg_button(
    svg_path: &str,
    size: f32,
    color: impl Into<gpui::Hsla>,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let svg_path = svg_path.to_string(); // Clone the string to own it
    let color = color.into(); // Convert color to owned type

    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size)) // Add padding around SVG
        .rounded_md()
        .cursor_pointer()
        .hover(|this| this.bg(rgba(0x00000010))) // Light hover effect
        .child(
            svg()
                .path(svg_path) // Now using owned string
                .size(px(size))
                .text_color(color), // Now using owned color
        )
}

struct Main {
    address_state: Entity<InputState>,
    webview: Entity<WebView>,
}

impl Main {
    fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let background_executor = cx.background_executor().clone();

        let func_registry = FuncRegistry::builder()
            .with_spawner(move |fut| {
                background_executor.spawn(fut).detach();
            })
            .register("toUppercase", |value: String| value.to_uppercase())
            .register("addInt", |a: i32, b: i32| a + b)
            .register("parseInt", |value: String| value.parse::<i32>())
            .register_async("sleep", |millis: u64| async move {
                Timer::after(Duration::from_millis(millis)).await;
                "ok"
            })
            .register("emit", |frame: Frame| {
                #[derive(Debug, Serialize)]
                struct Message {
                    event: String,
                    data: String,
                }

                frame.emit(Message {
                    event: "custom".to_string(),
                    data: "ok".to_string(),
                });
            })
            .build();

        cx.new(|cx| {
            let url = "https://vercel.com";

            // create webview
            let webview = WebView::with_func_registry(url, func_registry.clone(), window, cx);

            window
                .subscribe(&webview, cx, |_, event: &TitleChangedEvent, window, _| {
                    window.set_window_title(&event.title);
                })
                .detach();

            // create address input
            let address_state = cx.new(|cx| InputState::new(window, cx).default_value(url));

            window
                .subscribe(&address_state, cx, {
                    let webview = webview.clone();
                    move |state, event: &InputEvent, _, cx| {
                        if let InputEvent::PressEnter { .. } = event {
                            let url = state.read(cx).value();
                            webview.read(cx).browser().load_url(url);
                        }
                    }
                })
                .detach();

            Self {
                address_state,
                webview,
            }
        })
    }
}

impl Render for Main {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(
                div()
                    .border_1()
                    .border_color(rgba(0xd3d9d92b))
                    .rounded_xl()
                    .bg(rgba(0x0404055e))
                    .size_full()
                    .child(
                        div()
                            .pl(px(84.)) // Left padding to clear traffic lights
                            .py(px(10.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        // Back button
                                        svg_button("back.svg", 14.0, rgb(0xf2f2f2), |_, _| {
                                            println!("Back clicked!")
                                        }),
                                    )
                                    .child(
                                        // Forward button
                                        svg_button(
                                            "forward.svg",
                                            14.0,
                                            rgba(0xd3d9d92b),
                                            |_, _| println!("Forward clicked!"),
                                        ),
                                    )
                                    .child(
                                        // Refresh button
                                        svg_button("rotate-cw.svg", 12.0, rgb(0xf2f2f2), |_, _| {
                                            println!("Refresh clicked!")
                                        }),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .border_1()
                                            .border_color(rgba(0xd3d9d92b))
                                            .rounded_md()
                                            .h_8()
                                            .w_64()
                                            .items_center()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .px_3()
                                                    .h_full()
                                                    .w_full()
                                                    .child(
                                                        svg()
                                                            .path("vercel.svg")
                                                            .size(px(10.0))
                                                            .text_color(rgb(0xfefefe)),
                                                    )
                                                    .child(
                                                        TextInput::new(&self.address_state)
                                                            .text_color(rgb(0xd1d1d1))
                                                            .text_xs()
                                                            .border_0(),
                                                    )
                                                    .child(
                                                        svg()
                                                            .path("close.svg")
                                                            .size(px(10.0))
                                                            .text_color(rgba(0xffffffb3)),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_1()
                                            .py_1()
                                            .bg(linear_gradient(
                                                150.,
                                                linear_color_stop(rgba(0x2e2e2e1c), 0.05), // transparent
                                                linear_color_stop(rgba(0x6161621c), 0.85), // Very dark/black
                                            ))
                                            .border_1()
                                            .border_color(rgba(0xd3d9d92b))
                                            .rounded_md()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                svg()
                                                    .path("plus.svg")
                                                    .size(px(12.0))
                                                    .text_color(rgb(0xf2f2f2)),
                                            ),
                                    ),
                            ),
                    )
                    .child(self.webview.clone()),
            )
            .children(Root::render_modal_layer(window, cx))
    }
}

fn run() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
            if cfg!(target_os = "linux") {
                cx.spawn(async move |cx| {
                    let (tx, rx) = flume::unbounded();

                    cx.background_spawn(async move {
                        let mut timer = Timer::interval(Duration::from_millis(1000 / 60));
                        while timer.next().await.is_some() {
                            _ = tx.send_async(()).await;
                        }
                    })
                    .detach();

                    while rx.recv_async().await.is_ok() {
                        wef::do_message_work();
                    }
                })
                .detach();
            }

            gpui_component::init(cx);

            let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: gpui::WindowBackgroundAppearance::Blurred,
                    titlebar: Some(gpui::TitlebarOptions {
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(16.0), px(18.0))), // Custom position
                        title: Option::Some(SharedString::from("Browser App")),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let main = Main::new(window, cx);
                    cx.new(|cx| Root::new(main.into(), window, cx))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    wef::launch(Settings::new(), run);
    Ok(())
}
