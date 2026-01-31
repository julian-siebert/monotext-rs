use crate::render::Render;

mod render;

/// Rendering configuration for plain-text document output.
///
/// This configuration defines the physical page layout and
/// page numbering behavior used during rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// Total number of text lines per page, including header and footer.
    pub page_height: usize,

    /// Maximum number of characters per line.
    pub page_width: usize,

    /// Number of pages rendered using Roman numerals (i, ii, iii, ...).
    ///
    /// This is typically used for front matter such as title pages,
    /// abstracts, and tables of contents.
    pub roman_pages: usize,
}

/// A fully structured document ready for rendering.
///
/// `Document` represents the complete logical structure of a text-based
/// specification or technical document. It is independent of any output
/// format and contains no layout information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Main document title.
    pub title: String,

    /// Optional subtitle displayed directly below the title.
    pub subtitle: Option<String>,

    /// Publication date of the document.
    pub date: time::Date,

    /// List of document authors.
    pub authors: Vec<Author>,

    /// Institutions referenced by authors.
    pub institutions: Vec<Institution>,

    /// Abstract or executive summary of the document.
    pub r#abstract: String,

    /// Main document body content.
    pub content: Vec<Content>,
}

/// Metadata describing a document author.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author {
    /// Given name.
    pub first_name: Option<String>,

    /// Middle name or initials.
    pub middle_name: Option<String>,

    /// Family name / surname.
    pub last_name: String,

    /// Optional academic or professional title (e.g., "Dr.", "Prof.").
    pub title: Option<String>,

    /// Contact email address.
    pub email: Option<String>,

    /// Institutional affiliation.
    pub affiliation: Option<Institution>,

    /// Contact phone number.
    pub phone: Option<String>,

    /// Postal address.
    pub address: Option<String>,
}

/// Institutional affiliation metadata.
///
/// This structure is intentionally flexible to support academic,
/// governmental, and corporate institutions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Institution {
    pub name: String,
    pub department: Option<String>,
    pub street: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub code: Option<String>,
}

impl Document {
    /// Render the document as paginated plain text.
    ///
    /// This method performs layout, pagination, header/footer generation,
    /// and text wrapping according to the provided [`Config`].
    ///
    /// # Returns
    ///
    /// A single `String` containing the fully rendered document,
    /// including page breaks and control characters.
    pub fn render(&self, config: Config) -> String {
        let mut render = Render::new(config.clone(), self);

        render.render_cover_page();

        render.new_page();

        let abstract_lines: Vec<String> = textwrap::wrap(&self.r#abstract, config.page_width)
            .into_iter()
            .map(|s| s.into_owned())
            .collect();

        for line in abstract_lines {
            render.push_line(&line);
        }

        render.push_line("");

        render.write_contents(&self.content);

        render.finish()
    }
}

/// Logical content elements of a document.
///
/// `Content` is purely structural and contains no pagination
/// or layout state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// A free-flowing paragraph of text.
    Paragraph { text: String },

    /// A section heading.
    Headline {
        text: String,

        /// Horizontal indentation in spaces.
        indent: usize,

        /// Horizontal alignment of the headline text.
        position: ContentPosition,
    },

    /// An unordered (bulleted) list.
    UnsortedList {
        contents: Vec<Content>,

        /// Whether list items are rendered without blank lines between them.
        compact: bool,
    },

    /// An ordered (numbered or lettered) list.
    OrderedList {
        contents: Vec<Content>,

        /// Starting ordinal value.
        start: u8,

        /// Type of ordinal numbering used.
        r#type: OrderedListType,

        /// Whether list items are rendered without blank lines between them.
        compact: bool,
    },
}

/// Horizontal alignment options for content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentPosition {
    Left,
    Center,
    Right,
}

/// Enumeration of ordered list numbering styles.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum OrderedListType {
    /// (a, b, c, ...)
    LowerCaseLetters = 1,
    /// (A, B, C, ...)
    UpperCaseLetters = 2,
    /// (1, 2, 3, ...)
    DecimalNumbers = 3,
    /// (i, ii, iii, ...)
    LowercaseRomanNumerals = 4,
    /// (I, II, III, ...)
    UppercaseRomanNumerals = 5,
}

impl Content {
    pub(crate) fn render_lines(&self, width: usize) -> Vec<String> {
        match self {
            Content::Paragraph { text } => {
                let opts = textwrap::Options::new(width);
                let mut lines: Vec<String> = textwrap::wrap(text, &opts)
                    .into_iter()
                    .map(|s| s.into_owned())
                    .collect();
                lines.push(String::new());
                lines
            }
            Content::Headline {
                text,
                indent,
                position,
            } => {
                let line = match position {
                    ContentPosition::Left => format!("{:indent$}{}", "", text, indent = *indent),
                    ContentPosition::Center => {
                        let padding = (width.saturating_sub(text.len())) / 2;
                        format!("{:padding$}{}", "", text, padding = padding)
                    }
                    ContentPosition::Right => {
                        let padding = width.saturating_sub(text.len());
                        format!("{:padding$}{}", "", text, padding = padding)
                    }
                };
                vec![line, String::new()]
            }
            Content::UnsortedList { contents, compact } => {
                let mut lines = Vec::new();
                for c in contents {
                    for l in c.render_lines(width) {
                        lines.push(format!("* {}", l));
                    }
                    if !*compact {
                        lines.push(String::new());
                    }
                }
                lines
            }
            Content::OrderedList {
                contents,
                start,
                r#type,
                compact,
            } => {
                let mut lines = Vec::new();
                for (i, c) in contents.iter().enumerate() {
                    let index = match r#type {
                        OrderedListType::LowerCaseLetters => {
                            ((i + *start as usize) % 26 + b'a' as usize) as u8 as char
                        }
                        OrderedListType::UpperCaseLetters => {
                            ((i + *start as usize) % 26 + b'A' as usize) as u8 as char
                        }
                        OrderedListType::DecimalNumbers => (i + *start as usize + 1)
                            .to_string()
                            .chars()
                            .next()
                            .unwrap_or('1'),
                        _ => '?', // simplified for now
                    };
                    for l in c.render_lines(width) {
                        lines.push(format!("{} {}", index, l));
                    }
                    if !*compact {
                        lines.push(String::new());
                    }
                }
                lines
            }
        }
    }
}
