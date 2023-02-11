//! Example: File Explorer
//! -------------------------
//!
//! This is a fun little desktop application that lets you explore the file system.
//!
//! This example is interesting because it's mixing filesystem operations and GUI, which is typically hard for UI to do.
use log;
use dioxus::prelude::{*, dioxus_elements::button};

fn main() {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    dioxus::desktop::launch_cfg(App, |c| {
        c.with_window(|w| {
            w.with_resizable(true).with_inner_size(
                dioxus::desktop::wry::application::dpi::LogicalSize::new(400.0, 800.0),
            )
        })
    });
}

static App: Component<()> = |cx| {
    rsx!(cx, div {
        style { [include_str!("./style.css")] }
        main {
            rsx!(
                div {
                    button {
                        onclick: move |_| log::info!("click btn!"), 
                        "start"
                    }
                }
            )
        }

    })
};
