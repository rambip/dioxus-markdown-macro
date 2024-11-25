use std::{iter::Peekable, vec};
use regex::Regex;

use dioxus_rsx::{BodyNode, TemplateBody, RsxBlock};
use pulldown_cmark::{Alignment, Event, Options, Parser, Tag};
use quote::quote;
use syn::{
    Ident,
    __private::Span,
    parse_macro_input, parse_quote, parse_str, LitStr, 
};

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use proc_macro::TokenStream;
use std::{fs, path::PathBuf};

#[proc_macro]
pub fn md_page(input: TokenStream) -> TokenStream {
    let file_path = parse_macro_input!(input as LitStr);

    // Get the manifest directory of the crate using the macro
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    // Combine with the provided path
    let full_path = PathBuf::from(manifest_dir)
        .join("src")
        .join(file_path.value());

    let content = fs::read_to_string(full_path).unwrap();
    let items = extract_items(&content);

    let children: Vec<BodyNode> = items
        .into_iter()
        .flat_map(|x| x.to_body_nodes())
        .collect();

    let template_body = TemplateBody::new(children);


    quote!(
        #template_body
    ).into()
}

fn extract_items(text: &str) -> Vec<Item> {
    let re = Regex::new(r"(?sU)\{\{(.*)\}\}").unwrap();
        
    // Collect all matches into a vector
    //re.captures_iter(text)
    //    .map(|cap| Item::new(&cap[1], &cap[2]))
    //    .collect()    let mut last_end = 0;

    let mut last_end = 0;
    let mut result = Vec::new();

    for capture in re.captures_iter(&text) {
        // Add text before the match to outside_texts
        if let Some(pre_match) = text.get(last_end..capture.get(0).unwrap().start()) {
            if !pre_match.trim().is_empty() {
                result.push(Item {
                    content: pre_match.trim().to_string(),
                    content_type: ItemType::Md
                });
            }
        }
        
        // Add text inside braces as rsx
        result.push(Item {
            content: capture[1].to_string(),
            content_type: ItemType::Rsx,
        });
        
        last_end = capture.get(0).unwrap().end();
    }
    
    // Add any remaining text after last match to outside_texts
    if last_end < text.len() {
        if let Some(post_match) = text.get(last_end..) {
            if !post_match.trim().is_empty() {
                result.push(Item {
                    content: post_match.trim().to_string(),
                    content_type: ItemType::Md
                });
            }
        }
    }
    
    result
}

#[derive(Debug, PartialEq)]
enum ItemType {
    Rsx,
    Md,
}

#[derive(Debug, PartialEq)]
struct Item {
    content: String,
    content_type: ItemType
}


impl Item {
    fn to_body_nodes(&self) -> Vec<BodyNode> {
        match self.content_type {
            ItemType::Md => {
                parse_md(&self.content).expect("malformed md")
            }
            ItemType::Rsx => {
                let tokens: TokenStream = &self.content.parse().expect("invalid bracketing");
                let block = parse_macro_input!(tokens with RsxBlock::parse_children).expect("malformed rsx");
                block.children
            }
        }

    }
}


fn parse_md(markdown: &str) -> syn::Result<Vec<BodyNode>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let mut parser = Parser::new_ext(markdown, options);

    let mut rsx_parser = RsxMarkdownParser {
        element_stack: vec![],
        root_nodes: vec![],
        current_table: vec![],
        in_table_header: false,
        iter: parser.by_ref().peekable(),
        phantom: std::marker::PhantomData,
    };
    rsx_parser.parse()?;
    while !rsx_parser.element_stack.is_empty() {
        rsx_parser.end_node();
    }

    Ok(rsx_parser.root_nodes)
}

struct RsxMarkdownParser<'a, I: Iterator<Item = Event<'a>>> {
    element_stack: Vec<BodyNode>,
    root_nodes: Vec<BodyNode>,

    current_table: Vec<Alignment>,
    in_table_header: bool,

    iter: Peekable<I>,

    phantom: std::marker::PhantomData<&'a ()>,
}


impl<'a, I: Iterator<Item = Event<'a>>> RsxMarkdownParser<'a, I> {
    fn parse(&mut self) -> syn::Result<()> {
        while let Some(event) = self.iter.next() {
            self.parse_event(event)?;
        }
        Ok(())
    }

    fn parse_event(&mut self, event: Event) -> syn::Result<()> {
        match event {
            pulldown_cmark::Event::Start(start) => {
                self.start_element(start)?;
            }
            pulldown_cmark::Event::End(_) => self.end_node(),
            pulldown_cmark::Event::Text(text) => {
                let text = escape_text(&text);
                self.create_node(BodyNode::Text(parse_quote!(#text)));
            }
            pulldown_cmark::Event::Code(code) => {
                let code = escape_text(&code);
                self.create_node(parse_quote! {
                    code {
                        #code
                    }
                })
            }
            pulldown_cmark::Event::Html(_) => {}
            pulldown_cmark::Event::FootnoteReference(_) => {}
            pulldown_cmark::Event::SoftBreak => {}
            pulldown_cmark::Event::HardBreak => {}
            pulldown_cmark::Event::Rule => self.create_node(parse_quote! {
                hr {}
            }),
            pulldown_cmark::Event::TaskListMarker(value) => {
                self.write_checkbox(value);
            }
        }
        Ok(())
    }

    fn write_checkbox(&mut self, checked: bool) {
        let type_value = if checked { "true" } else { "false" };
        self.create_node(parse_quote! {
            input {
                r#type: "checkbox",
                value: #type_value,
            }
        })
    }

    fn take_code_or_text(&mut self) -> String {
        let mut current_text = String::new();
        while let Some(pulldown_cmark::Event::Code(text) | pulldown_cmark::Event::Text(text)) =
            self.iter.peek()
        {
            current_text += text;
            let _ = self.iter.next().unwrap();
        }
        current_text
    }

    fn write_text(&mut self) {
        loop {
            match self.iter.peek() {
                Some(pulldown_cmark::Event::Text(text)) => {
                    let mut all_text = text.to_string();

                    // Take the text or code event we just inserted
                    let _ = self.iter.next().unwrap();

                    // If the next block after this is a code block, insert the space in the text before the code block
                    if let Some(pulldown_cmark::Event::Code(_)) = self.iter.peek() {
                        all_text.push(' ');
                    }
                    let all_text = escape_text(&all_text);

                    let text = BodyNode::Text(parse_quote!(#all_text));
                    self.create_node(text);
                }
                Some(pulldown_cmark::Event::Code(code)) => {
                    let code = code.to_string();
                    let code = escape_text(&code);
                    self.create_node(parse_quote! {
                        code {
                            #code
                        }
                    });

                    // Take the text or code event we just inserted
                    let _ = self.iter.next().unwrap();
                }
                _ => return,
            }
        }
    }

    fn take_text(&mut self) -> String {
        let mut current_text = String::new();
        // pulldown_cmark will create a new text node for each newline. We insert a space
        // between each newline to avoid two lines being rendered right next to each other.
        let mut insert_space = false;
        while let Some(pulldown_cmark::Event::Text(text)) = self.iter.peek() {
            if insert_space {
                current_text.push(' ');
            }
            current_text += text;
            insert_space = true;
            let _ = self.iter.next().unwrap();
        }
        current_text
    }

    fn start_element(&mut self, tag: Tag) -> syn::Result<()> {
        match tag {
            Tag::Paragraph => {
                self.start_node(parse_quote! {
                    p {}
                });
                self.write_text();
            }
            Tag::Heading(level, _, _) => {
                let text = self.take_text();
                let anchor: String = text
                    .trim()
                    .to_lowercase()
                    .chars()
                    .filter_map(|char| match char {
                        '-' | 'a'..='z' | '0'..='9' => Some(char),
                        ' ' | '_' => Some('-'),
                        _ => None,
                    })
                    .collect();
                let fragment = format!("#{}", anchor);
                let element_name = match level {
                    pulldown_cmark::HeadingLevel::H1 => Ident::new("h1", Span::call_site()),
                    pulldown_cmark::HeadingLevel::H2 => Ident::new("h2", Span::call_site()),
                    pulldown_cmark::HeadingLevel::H3 => Ident::new("h3", Span::call_site()),
                    pulldown_cmark::HeadingLevel::H4 => Ident::new("h4", Span::call_site()),
                    pulldown_cmark::HeadingLevel::H5 => Ident::new("h5", Span::call_site()),
                    pulldown_cmark::HeadingLevel::H6 => Ident::new("h6", Span::call_site()),
                };
                let anchor = escape_text(&anchor);
                let fragment = escape_text(&fragment);
                let text = escape_text(&text);
                let element = parse_quote! {
                    #element_name {
                        id: #anchor,
                        a {
                            href: #fragment,
                            class: "header",
                            #text
                        }
                    }
                };
                self.start_node(element);
            }
            Tag::BlockQuote => {
                self.start_node(parse_quote! {
                    blockquote {}
                });
                self.write_text();
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Indented => None,
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        (!lang.is_empty()).then_some(lang)
                    }
                };
                let raw_code = self.take_code_or_text();

                if lang.as_deref() == Some("inject-dioxus") {
                    self.start_node(parse_str::<BodyNode>(&raw_code).unwrap());
                } else {
                    let code = transform_code_block(raw_code)?;

                    let ss = SyntaxSet::load_defaults_newlines();
                    let ts = ThemeSet::load_defaults();

                    let theme = &ts.themes["base16-ocean.dark"];
                    let syntax = ss.find_syntax_by_extension("rs").unwrap();
                    let html = escape_text(
                        &syntect::html::highlighted_html_for_string(&code, &ss, syntax, theme)
                            .unwrap(),
                    );
                    self.start_node(parse_quote!{
                        div {
                            style: "position: relative;",
                            div {
                                dangerous_inner_html: #html
                            }
                            button {
                                style: "position: absolute; top: 0; right: 0; background: rgba(0, 0, 0, 0.75); color: white; border: 1px solid white; padding: 0.25em;",
                                "onclick": "navigator.clipboard.writeText(this.previousElementSibling.innerText)",
                                "Copy"
                            }
                        }
                    });
                }
            }
            Tag::List(first) => {
                let name = match first {
                    Some(_) => Ident::new("ol", Span::call_site()),
                    None => Ident::new("ul", Span::call_site()),
                };
                self.start_node(parse_quote! {
                    #name {}
                })
            }
            Tag::Item => self.start_node(parse_quote! {
                li {}
            }),
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(alignments) => {
                self.current_table = alignments;
                self.start_node(parse_quote! {
                    table {}
                })
            }
            Tag::TableHead => {
                self.in_table_header = true;
                self.start_node(parse_quote! {
                    thead {}
                })
            }
            Tag::TableRow => self.start_node(parse_quote! {
                tr {}
            }),
            Tag::TableCell => {
                let name = if self.in_table_header { "th" } else { "td" };
                let ident = Ident::new(name, Span::call_site());
                self.start_node(parse_quote! {
                    #ident {}
                })
            }
            Tag::Emphasis => self.start_node(parse_quote! {
                em {}
            }),
            Tag::Strong => self.start_node(parse_quote! {
                strong {}
            }),
            Tag::Strikethrough => self.start_node(parse_quote! {
                s {}
            }),
            Tag::Link(ty, dest, title) => {
                let without_extension = dest.trim_end_matches(".md");
                let without_index = without_extension.trim_end_matches("/index");

                let href = match ty {
                    pulldown_cmark::LinkType::Email => format!("mailto:{}", without_index),
                    _ => {
                        if dest.starts_with("http") || dest.starts_with("https") {
                            dest.to_string()
                        } else {
                            without_index.to_string()
                        }
                    }
                };
                let href = escape_text(&href);
                let title = escape_text(&title);
                let title_attr = if !title.is_empty() {
                    quote! {
                        title: #title,
                    }
                } else {
                    quote! {}
                };

                self.start_node(parse_quote! {
                    a {
                        href: #href,
                        #title_attr
                        #title
                    }
                });

                self.write_text();
            }
            Tag::Image(_, dest, title) => {
                let alt = escape_text(&self.take_text());
                let dest: &str = &dest;
                let title = escape_text(&title);

                #[cfg(not(feature = "manganis"))]
                let url: syn::Expr = {
                    let dest = escape_text(dest);
                    syn::parse_quote!(#dest)
                };
                #[cfg(feature = "manganis")]
                let url: syn::Expr = {
                    let remote_image = dest.starts_with("http:") || dest.starts_with("https:");
                    if remote_image {
                        let dest = escape_text(dest);
                        syn::parse_quote!(#dest)
                    } else {
                        syn::parse_quote! { manganis::mg!(file(#dest)) }
                    }
                };

                self.start_node(parse_quote! {
                    img {
                        src: #url,
                        alt: #alt,
                        title: #title,
                    }
                })
            }
        }
        Ok(())
    }

    fn start_node(&mut self, node: BodyNode) {
        self.element_stack.push(node);
    }

    fn end_node(&mut self) {
        if let Some(node) = self.element_stack.pop() {
            match self.last_mut() {
                Some(BodyNode::Element(element)) => {
                    element.children.push(node);
                }
                None => {
                    self.root_nodes.push(node);
                }
                _ => {}
            }
        }
    }

    fn create_node(&mut self, node: BodyNode) {
        // Find the list of elements we should add the node to
        let element_list = match self.last_mut() {
            Some(BodyNode::Element(element)) => &mut element.children,
            None => &mut self.root_nodes,
            _ => return,
        };

        // If the last element is a text node, we can just join the text nodes together with a space
        if let (Some(BodyNode::Text(last_text)), BodyNode::Text(new_text)) =
            (element_list.last_mut(), &node)
        {
            last_text
                .input
                .formatted_input
                .push_ifmt(new_text.input.formatted_input.clone());
        } else {
            element_list.push(node);
        }
    }

    fn last_mut(&mut self) -> Option<&mut BodyNode> {
        self.element_stack.last_mut()
    }
}

fn transform_code_block(code_contents: String) -> syn::Result<String> {
    let segments = code_contents.split("{{#");
    let mut output = String::new();
    for segment in segments {
        if let Some((plugin, after)) = segment.split_once("}}") {
            if plugin.starts_with("include") {
                output += &resolve_extension(plugin)?;
                output += after;
            }
        } else {
            output += segment;
        }
    }
    Ok(output)
}

fn resolve_extension(ext: &str) -> syn::Result<String> {
    if let Some(file) = ext.strip_prefix("include") {
        let file = file.trim();
        let mut segment = None;
        let file = if let Some((file, file_segment)) = file.split_once(':') {
            segment = Some(file_segment);
            file
        } else {
            file
        };
        let result = std::fs::read_to_string(file).map_err(|e| {
            syn::Error::new(
                Span::call_site(),
                format!("Failed to read file {}: {}", file, e),
            )
        })?;
        if let Some(segment) = segment {
            // get the text between lines with ANCHOR: segment and ANCHOR_END: segment
            let lines = result.lines();
            let mut output = String::new();
            let mut in_segment: bool = false;
            // normalize indentation to the first line
            let mut first_line_indent = 0;
            for line in lines {
                if let Some((_, remaining)) = line.split_once("ANCHOR:") {
                    if remaining.trim() == segment {
                        in_segment = true;
                        first_line_indent = line.chars().take_while(|c| c.is_whitespace()).count();
                    }
                } else if let Some((_, remaining)) = line.split_once("ANCHOR_END:") {
                    if remaining.trim() == segment {
                        in_segment = false;
                    }
                } else if in_segment {
                    for (_, char) in line
                        .chars()
                        .enumerate()
                        .skip_while(|(i, c)| *i < first_line_indent && c.is_whitespace())
                    {
                        output.push(char);
                    }
                    output += "\n";
                }
            }
            if output.ends_with('\n') {
                output.pop();
            }
            Ok(output)
        } else {
            Ok(result)
        }
    } else {
        todo!("Unknown extension: {}", ext);
    }
}

fn escape_text(text: &str) -> String {
    text.replace('{', "{{").replace('}', "}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse(){
        let content = "{{}} after";
        let items = extract_items(content);
        assert_eq!(
            items,
            vec![Item {content: "".to_string(), content_type: ItemType::Rsx}, Item { content: "after".to_string(), content_type: ItemType::Md }]
        );
    }
    #[test]
    fn test_parse_multiple(){
        let content = "a {{}}";
        let items = extract_items(content);
        assert_eq!(
            items,
            vec![
            Item { content: "a".to_string(), content_type: ItemType::Md},
            Item { content: "".to_string(), content_type: ItemType::Rsx},
            ]
        );
    }
    #[test]
    fn test_macro(){
        let content = r#"
        stuff

        {{
            Link {
                to: Route::PerfectClearPage { },
                "Go to Page"
            }
        }}

        ";
        "#;
        let _items = extract_items(content);
    }
}
