use anyhow::anyhow;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::{Content, ContentPosition, Document, OrderedListType};

pub fn markdown_to_document(md: &str) -> anyhow::Result<Document> {
    let (meta_yaml, body) = split_frontmatter(md)?;
    let mut doc: Document = serde_yaml::from_str(meta_yaml)?;

    let content = parse_markdown(body)?;
    doc.content = content;

    Ok(doc)
}

fn split_frontmatter(input: &str) -> anyhow::Result<(&str, &str)> {
    let mut parts = input.splitn(3, "---");

    parts.next();

    let yaml = parts.next().ok_or(anyhow!("missing yaml"))?;
    let body = parts.next().ok_or(anyhow!("missing body"))?;

    Ok((yaml.trim(), body.trim()))
}

enum ListBuilder {
    Unordered(Vec<Content>),
    Ordered(Vec<Content>, u8),
}

fn parse_markdown(md: &str) -> anyhow::Result<Vec<Content>> {
    let parser = Parser::new(md);

    let mut out = Vec::new();
    let mut text = String::new();

    let mut list_stack: Vec<ListBuilder> = vec![];

    for ev in parser {
        match ev {
            Event::Start(Tag::Paragraph) => {
                text.clear();
            }

            Event::End(TagEnd::Paragraph) => {
                push_paragraph(&mut out, &mut list_stack, &text);
            }

            Event::Start(Tag::Heading { .. }) => {
                text.clear();

                // heading text collected until End
                // handled below
            }

            Event::End(TagEnd::Heading(level)) => {
                let indent = (level as usize - 1) * 2;

                out.push(Content::Headline {
                    text: text.trim().to_string(),
                    indent,
                });
            }

            Event::Text(t) => {
                push_text(&mut text, &t);
            }

            Event::Code(t) => {
                push_text(&mut text, &t);
            }

            // links → ignore URL, keep text
            Event::Start(Tag::Link { .. }) => {}
            Event::End(TagEnd::Link { .. }) => {}

            // unordered list
            Event::Start(Tag::List(None)) => {
                list_stack.push(ListBuilder::Unordered(vec![]));
            }

            // ordered list
            Event::Start(Tag::List(Some(start))) => {
                list_stack.push(ListBuilder::Ordered(vec![], start as u8));
            }

            Event::End(TagEnd::List(_)) => {
                finalize_list(&mut out, &mut list_stack);
            }

            Event::Start(Tag::Item) => {
                text.clear();
            }

            Event::End(TagEnd::Item) => {
                let item = Content::Paragraph {
                    text: text.trim().to_string(),
                };

                if let Some(builder) = list_stack.last_mut() {
                    match builder {
                        ListBuilder::Unordered(v) | ListBuilder::Ordered(v, _) => v.push(item),
                    }
                }
            }

            _ => {}
        }
    }

    Ok(out)
}

fn push_text(buf: &mut String, s: &str) {
    if !buf.ends_with(' ') && !buf.is_empty() {
        buf.push(' ');
    }
    buf.push_str(s);
}

fn push_paragraph(out: &mut Vec<Content>, stack: &mut Vec<ListBuilder>, text: &str) {
    let para = Content::Paragraph {
        text: text.trim().to_string(),
    };

    if let Some(builder) = stack.last_mut() {
        match builder {
            ListBuilder::Unordered(v) | ListBuilder::Ordered(v, _) => v.push(para),
        }
    } else {
        out.push(para);
    }
}

fn finalize_list(out: &mut Vec<Content>, stack: &mut Vec<ListBuilder>) {
    if let Some(builder) = stack.pop() {
        let content = match builder {
            ListBuilder::Unordered(v) => Content::UnsortedList {
                contents: v,
                compact: false,
            },

            ListBuilder::Ordered(v, start) => Content::OrderedList {
                contents: v,
                start,
                r#type: OrderedListType::DecimalNumbers,
                compact: false,
            },
        };

        if let Some(parent) = stack.last_mut() {
            match parent {
                ListBuilder::Unordered(v) | ListBuilder::Ordered(v, _) => v.push(content),
            }
        } else {
            out.push(content);
        }
    }
}
