rsx! { 

    h1 { id: "hello-world",
        a { href: "#hello-world", class: "header", "Hello world" }
    }
    hr {}
    p {
        strong { "bold" }
    }
    blockquote {
        p { "quote" }
    }
    Greet { name: "Dioxus" }
    for i in 0..10 {
        "{i}"
    }
 }