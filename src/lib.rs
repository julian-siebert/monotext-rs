use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::render::Render;

mod render;

#[cfg(feature = "md")]
pub mod md;

#[cfg(feature = "pdf")]
pub mod pdf;

#[cfg(feature = "serde")]
mod serde_intern;

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
}

/// A fully structured document ready for rendering.
///
/// `Document` represents the complete logical structure of a text-based
/// specification or technical document. It is independent of any output
/// format and contains no layout information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    /// Main document title.
    pub title: String,

    /// Optional subtitle displayed directly below the title.
    pub subtitle: Option<String>,

    /// Publication date of the document.
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "crate::serde_intern::deserialize")
    )]
    pub date: time::Date,

    /// List of document authors.
    #[cfg_attr(feature = "serde", serde(default))]
    pub authors: Vec<Author>,

    /// Institutions referenced by authors.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub institutions: Vec<Institution>,

    /// Abstract or executive summary of the document.
    pub r#abstract: String,

    /// License text
    pub license: Option<String>,

    /// Main document body content.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub content: Vec<Content>,
}

/// Metadata describing a document author.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

pub(crate) struct ContentTable {
    headlines: HashMap<u16, (usize, String)>,
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
    pub fn render(mut self, config: Config) -> String {
        self.title = self.title.to_uppercase();
        self.subtitle = self.subtitle.map(|s| s.to_uppercase());

        let mut content_table = ContentTable {
            headlines: HashMap::new(),
        };

        let mut i = 0;
        for c in &self.content {
            match c {
                Content::Headline { text, indent } => {
                    content_table.headlines.insert(i, (*indent, text.into()));
                }
                _ => {}
            }
            i += 1;
        }

        let content_table = Arc::new(Mutex::new(content_table));

        let mut render = Render::new(config.clone(), &self);

        render.render_cover_pages();

        render.new_page();

        render.write_header();

        render.push_centered("ABSTRACT");
        render.push_blank_lines(3);

        let abstract_lines: Vec<String> = textwrap::wrap(&self.r#abstract, config.page_width)
            .into_iter()
            .map(|s| s.into_owned())
            .collect();

        for line in abstract_lines {
            render.push_line(&line);
        }

        render.push_blank_lines(3);

        if let Some(license_text) = &self.license {
            render.push_centered("LICENSE");
            render.push_blank_lines(3);

            let license_lines: Vec<String> = textwrap::wrap(license_text, config.page_width)
                .into_iter()
                .map(|s| s.into_owned())
                .collect();

            for line in license_lines {
                render.push_centered(&line);
            }
        }

        render.page_break();

        render.push_blank_lines(3);

        render.push_centered(&self.title);

        if let Some(sub) = &self.subtitle {
            render.push_blank_lines(1);
            render.push_centered(sub);
        }

        render.push_blank_lines(5);

        render.write_contents(&self.content);

        render.finish()
    }
}

/// Logical content elements of a document.
///
/// `Content` is purely structural and contains no pagination
/// or layout state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Content {
    /// A free-flowing paragraph of text.
    Paragraph { text: String },

    /// A section heading.
    Headline {
        text: String,

        /// Horizontal indentation in spaces.
        indent: usize,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContentPosition {
    Left,
    Center,
    Right,
}

/// Enumeration of ordered list numbering styles.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
