#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

use markdown_macro::md_page;

mod perfect_clear;
use perfect_clear::PerfectClear;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/perfect_clear")]
    PerfectClearPage {},
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn PerfectClearPage() -> Element {
    rsx! {
        PerfectClear {
            size_bloc: 30,

        }
    }
}

#[component]
fn Greet(name: String) -> Element {
    rsx! {
        "hello {name} !"
        "how are you ?"
    }
}


#[component]
fn Home() -> Element {
    md_page!("hello.md")
}
