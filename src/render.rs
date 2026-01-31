use std::fmt::Write;

use time::Month;

use crate::{Config, Content, Document};

pub struct Render<'a> {
    document: &'a Document,
    config: Config,
    pager: Pager,
}

impl<'a> Render<'a> {
    pub fn new(config: Config, document: &'a Document) -> Self {
        let mut render = Self {
            document,
            config: config.clone(),
            pager: Pager::new(&config),
        };

        render.write_header();
        render
    }

    pub fn write_contents(&mut self, contents: &Vec<Content>) {
        for content in contents {
            self.write_content(content);
        }
    }

    pub fn write_content(&mut self, content: &Content) {
        let lines = content.render_lines(self.config.page_width);
        for line in lines {
            self.push_line(&line);
        }
    }

    pub fn push_line(&mut self, line: &str) {
        if self.pager.line_in_page >= self.pager.page_height {
            self.page_break();
        }

        self.pager.out.push_str(line);
        self.pager.out.push('\n');
        self.pager.line_in_page += 1;
    }

    pub fn push_blank_lines(&mut self, n: usize) {
        for _ in 0..n {
            self.push_line("");
        }
    }

    pub fn push_centered(&mut self, text: &str) {
        let width = self.config.page_width;
        let text_width = unicode_width::UnicodeWidthStr::width(text);

        let total_pad = width.saturating_sub(text_width);
        let left_pad = (total_pad + 1) / 2;
        let line = format!("{:left_pad$}{}", "", text, left_pad = left_pad);

        self.push_line(&line);
    }

    pub fn render_cover_page(&mut self) {
        self.push_blank_lines(5);

        self.push_centered(&self.document.title);

        if let Some(sub) = &self.document.subtitle {
            self.push_blank_lines(1);
            self.push_centered(sub);
        }

        self.push_blank_lines(5);

        for author in &self.document.authors {
            let mut parts = Vec::new();

            if let Some(title) = &author.title {
                parts.push(title.as_str());
            }

            if let Some(first) = &author.first_name {
                parts.push(first.as_str());
            }

            if let Some(middle) = &author.middle_name {
                parts.push(middle.as_str());
            }

            parts.push(&author.last_name.as_str());

            if let Some(aff) = &author.affiliation {
                parts.push(" ");
                parts.push(&aff.name);
            }

            let full_name = parts.join(" ");
            self.push_centered(&full_name);
        }

        self.push_blank_lines(2);

        for inst in &self.document.institutions {
            self.push_centered(&inst.name);
        }

        let date = format!(
            "{} {} {}",
            self.document.date.day(),
            month_name(self.document.date.month()),
            self.document.date.year()
        );

        self.push_centered(&date);
    }

    pub fn new_page(&mut self) {
        self.pager.pad_to_page_end();
        self.pager.force_page_break();
    }

    fn page_break(&mut self) {
        self.pager.pad_to_page_end();
        for footer_line in self.footer_lines() {
            self.pager.out.push_str(&footer_line);
            self.pager.out.push('\n');
        }

        self.pager.out.push('\n');
        self.pager.out.push('\x0C');

        self.pager.page += 1;
        self.pager.line_in_page = 0;

        self.write_header();
    }

    fn write_header(&mut self) {
        for header_line in self.header_lines() {
            self.pager.out.push_str(&header_line);
            self.pager.out.push('\n');
            self.pager.line_in_page += 1;
        }
    }

    fn header_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if self.pager.page < 2 {
            return lines;
        }

        let date_str = format!(
            "{} {}",
            month_name(self.document.date.month()),
            self.document.date.year()
        );

        lines.push(format!(
            "{:>width$}",
            date_str,
            width = self.config.page_width
        ));

        lines.push(truncate_to_width_left(
            &self.document.title,
            self.config.page_width,
        ));

        lines.push(String::new());

        lines
    }

    fn footer_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        let page_label = if self.pager.page <= self.config.roman_pages {
            int_to_roman(self.pager.page as u32).to_lowercase()
        } else {
            self.pager.page.to_string()
        };

        let right = format!("[Page {}]", page_label);

        let authors_left = self
            .document
            .authors
            .iter()
            .map(|a| abbreviate(&a.last_name, 12))
            .collect::<Vec<_>>()
            .join(", ");

        let left_len = authors_left.chars().count();
        let right_len = right.chars().count();
        let mut line = String::new();

        if left_len + right_len >= self.config.page_width {
            let truncated = truncate_to_width_left(
                &authors_left,
                self.config.page_width.saturating_sub(right_len),
            );
            write!(line, "{}{}", truncated, right).ok();
        } else {
            let pad = self.config.page_width - left_len - right_len;
            write!(line, "{}{:pad$}{}", authors_left, "", right, pad = pad).ok();
        }

        lines.push(line);
        lines
    }

    pub fn finish(mut self) -> String {
        self.pager.pad_to_page_end();

        for footer_line in self.footer_lines() {
            self.pager.out.push_str(&footer_line);
            self.pager.out.push('\n');
        }

        self.pager.out
    }
}

pub struct Pager {
    pub page: usize,
    pub line_in_page: usize,
    pub page_height: usize,
    pub out: String,
}

impl Pager {
    pub fn new(config: &Config) -> Self {
        Self {
            page: 0,
            line_in_page: 0,
            page_height: config.page_height,
            out: String::new(),
        }
    }

    pub fn pad_to_page_end(&mut self) {
        while self.line_in_page < self.page_height {
            self.out.push('\n');
            self.line_in_page += 1;
        }
    }

    pub fn force_page_break(&mut self) {
        self.out.push('\x0C');
        self.page += 1;
        self.line_in_page = 0;
    }
}

fn truncate_to_width_left(s: &str, width: usize) -> String {
    let mut result = String::new();
    let mut count = 0usize;
    for ch in s.chars() {
        if count + 1 > width {
            break;
        }
        result.push(ch);
        count += 1;
    }
    result
}

fn abbreviate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated = truncate_to_width_left(s, max_len);
        truncated
    }
}

fn month_name(m: Month) -> &'static str {
    match m {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    }
}

fn int_to_roman(mut num: u32) -> String {
    if num == 0 {
        return "0".to_string();
    }
    let symbols = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut result = String::new();
    for &(value, symbol) in symbols.iter() {
        while num >= value {
            result.push_str(symbol);
            num -= value;
        }
    }
    result
}
