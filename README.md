# What it is

This work is an experimental macro that allows you to use markdown and rsx syntax together in order to define components for [dioxus](dioxuslabs.com)


# Example

Inside your main rust file (`src/main.rs`), use:
```rust
use dioxus::prelude::*;
use dioxus_markdown_macro::md_page;

fn main() {
    launch(App);
}

#[component]
fn Greet(name: String) -> Element {
    rsx!{
        "Hello {name}"
    }
}

pub fn App() -> Element {
    md_page!("demo.md")
}

```

And inside `src/demo.md`:

```md
# I like markdown

**styling**, _of course_

# But I love dioxus

{{
    // some dioxus code
    Greet {
        name: "dioxus"
    }
}}

```

# Issues

For now, the macro panics when there is an error and hot reload does not work at all.


# Inspiration

I stole some code from this repo: <https://github.com/DioxusLabs/include_mdbook/blob/main/mdbook-macro/src/rsx.rs>
