use pulldown_cmark::{Event, Parser, Tag};
use serde::Deserialize;

use crate::Content;

#[derive(Debug, Deserialize)]
struct MarkdownMeta {
    title: Option<String>,
    subtitle: Option<String>,
    date: Option<String>,
    authors: Option<Vec<MarkdownAuthor>>,
    institutions: Option<Vec<MarkdownInstitution>>,
}

#[derive(Debug, Deserialize)]
struct MarkdownAuthor {
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: String,
    title: Option<String>,
    affiliation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MarkdownInstitution {
    name: String,
}

fn markdown_to_content(md: &str) -> Vec<Content> {
    let parser = Parser::new(md);
    let mut contents = Vec::new();
    let mut current_paragraph = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level)) => {
                if !current_paragraph.is_empty() {
                    contents.push(Content::Paragraph {
                        text: current_paragraph.clone(),
                    });
                    current_paragraph.clear();
                }
            }
            Event::Text(text) => {
                current_paragraph.push_str(&text);
            }
            Event::End(Tag::Heading(level)) => {
                contents.push(Content::Headline {
                    text: current_paragraph.clone(),
                    indent: 0,
                    position: crate::ContentPosition::Left,
                });
                current_paragraph.clear();
            }
            Event::SoftBreak | Event::HardBreak => {
                current_paragraph.push('\n');
            }
            _ => {}
        }
    }

    if !current_paragraph.is_empty() {
        contents.push(Content::Paragraph {
            text: current_paragraph,
        });
    }

    contents
}

fn split_front_matter(input: &str) -> (String, String) {
    if input.starts_with("---") {
        let mut lines = input.lines();
        lines.next(); // skip first '---'
        let mut front = Vec::new();
        for line in &mut lines {
            if line.trim() == "---" {
                break;
            }
            front.push(line);
        }
        let front_str = front.join("\n");
        let body_str = lines.collect::<Vec<_>>().join("\n");
        (front_str, body_str)
    } else {
        ("".into(), input.into())
    }
}
