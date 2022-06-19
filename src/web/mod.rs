use hyper::Body;

const HTML_LOGIN: &'static str = include_str!("login.html");
const HTML_MAIN: &'static str = include_str!("main.html");

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Page {
    Login,
    Main,
}

/// HTML Body loader
pub struct HtmlBody {
    data: String,
}

impl HtmlBody {
    /// Constructor
    ///
    /// * `Page` - Enum representing the desired html page
    ///
    pub fn new(page: Page) -> Self {
        let data = match page {
            Page::Main => HTML_MAIN,
            Page::Login => HTML_LOGIN,
        };

        Self {
            data: data.to_string(),
        }
    }

    /// Inserts variables into the html
    ///
    /// * `key` - name of the variable
    /// * `var` - value of the variable
    ///
    pub fn var(mut self, key: impl AsRef<str>, var: impl AsRef<str>) -> Self {
        // todo: replace with more efficient method
        self.data = self.data.replace(
            &*format!("{{{{{}}}}}", key.as_ref()),
            &*format!("{}", var.as_ref()),
        );

        self
    }
}

impl Into<Body> for HtmlBody {
    fn into(self) -> Body {
        Body::from(self.data)
    }
}
