use monotext::{Author, Config, Content, Document, Institution};

fn main() {
    let long_paragraph = Content::Paragraph {
        text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
               Vestibulum sed turpis ac justo hendrerit ullamcorper. \
               Fusce vitae sapien vitae nunc imperdiet bibendum. \
               Curabitur efficitur diam non leo viverra, vel fermentum \
               massa venenatis. Sed vel odio sed elit viverra feugiat. \
               Maecenas volutpat est nec tellus pretium, at feugiat \
               turpis posuere. Vivamus rutrum leo sed sapien cursus, \
               at sollicitudin risus imperdiet. Praesent nec sem \
               imperdiet, cursus lorem sed, cursus magna. Aliquam \
               erat volutpat. Sed laoreet, justo ac blandit malesuada, \
               turpis libero sodales arcu, nec dignissim arcu sapien \
               nec magna. In vel ligula eget arcu ultricies viverra. \
               Integer fermentum felis ac nulla commodo, in imperdiet \
               justo fermentum. Proin a leo eu mi viverra hendrerit \
               a in libero. Cras in justo at leo scelerisque gravida. \
               Integer sed lectus ac sapien facilisis scelerisque. \
               Quisque dictum elit a sapien pretium, ac hendrerit \
               est sagittis. Donec imperdiet purus non ligula \
               suscipit, at convallis mi ultricies. Vivamus non \
               neque id eros malesuada volutpat. Suspendisse potenti. \
               Nullam ac nisi quis urna vehicula scelerisque. Morbi \
               fringilla erat ut urna ultrices, nec sodales sapien \
               eleifend. Aliquam erat volutpat. Integer euismod \
               lectus ac turpis fermentum, a sodales odio rhoncus. \
               Nam consequat sapien at ligula lacinia, sed vehicula \
               libero eleifend. Sed ac nisl justo. Donec tempor \
               sapien ut nulla sollicitudin, id facilisis nunc \
               suscipit. Phasellus eget magna vel leo aliquam \
               dictum. Fusce et risus a libero viverra ultrices."
            .to_string(),
    };
    let doc = Document {
        title: "TEST TITLE".into(),
        subtitle: None,
        date: time::Date::from_calendar_date(1981, time::Month::December, 1).unwrap(),
        authors: vec![
            Author {
                first_name: Some("Julian".into()),
                middle_name: None,
                last_name: "Siebert".into(),
                title: None,
                email: None,
                affiliation: Some(Institution {
                    name: "ICANN".into(),
                    department: None,
                    street: None,
                    postal_code: None,
                    city: None,
                    state: None,
                    country: None,
                    phone: None,
                    email: None,
                    website: None,
                    code: None,
                }),
                phone: None,
                address: None,
            },
            Author {
                first_name: Some("Jonas".into()),
                middle_name: None,
                last_name: "Löwendorf".into(),
                title: None,
                email: None,
                affiliation: Some(Institution {
                    name: "ICANN".into(),
                    department: None,
                    street: None,
                    postal_code: None,
                    city: None,
                    state: None,
                    country: None,
                    phone: None,
                    email: None,
                    website: None,
                    code: None,
                }),
                phone: None,
                address: None,
            },
        ],
        institutions: vec![],
        r#abstract: "This RFC specifies the Deez Nuts Protocol.".into(),
        content: vec![long_paragraph],
    };

    let txt = doc.render(Config {
        page_height: 57,
        page_width: 72,
        roman_pages: 0,
    });
    println!("{}", txt);
}
