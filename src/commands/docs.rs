//! `echidna docs` command implementation.
//!
//! Opens ChimeraX documentation in the browser.

use crate::error::{EchidnaError, Result};

/// Base URLs for ChimeraX documentation.
const USER_DOCS_URL: &str = "https://www.cgl.ucsf.edu/chimerax/docs/user/";
const DEV_DOCS_URL: &str = "https://www.cgl.ucsf.edu/chimerax/docs/devel/";
const API_DOCS_URL: &str = "https://www.cgl.ucsf.edu/chimerax/docs/devel/modules/";

/// Arguments for the docs command.
pub struct DocsArgs {
    /// Open developer documentation
    pub dev: bool,
    /// Open API reference
    pub api: bool,
    /// Search query
    pub query: Option<String>,
}

/// Execute the docs command.
pub fn execute(args: DocsArgs) -> Result<()> {
    let url = build_url(&args);

    println!("Opening: {}", url);

    open::that(&url).map_err(|e| {
        EchidnaError::Io(std::io::Error::other(format!("Failed to open browser: {}", e)))
    })?;

    Ok(())
}

/// Build the documentation URL based on arguments.
fn build_url(args: &DocsArgs) -> String {
    // Determine base URL
    let base_url = if args.api {
        API_DOCS_URL
    } else if args.dev {
        DEV_DOCS_URL
    } else {
        USER_DOCS_URL
    };

    // If search query provided, use Google site search
    if let Some(ref query) = args.query {
        let encoded_query = urlencoding::encode(query);
        let site = if args.dev || args.api {
            "cgl.ucsf.edu/chimerax/docs/devel"
        } else {
            "cgl.ucsf.edu/chimerax/docs/user"
        };
        format!(
            "https://www.google.com/search?q=site:{}+{}",
            site, encoded_query
        )
    } else {
        base_url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_docs_url() {
        let args = DocsArgs {
            dev: false,
            api: false,
            query: None,
        };
        assert_eq!(build_url(&args), USER_DOCS_URL);
    }

    #[test]
    fn test_dev_docs_url() {
        let args = DocsArgs {
            dev: true,
            api: false,
            query: None,
        };
        assert_eq!(build_url(&args), DEV_DOCS_URL);
    }

    #[test]
    fn test_api_docs_url() {
        let args = DocsArgs {
            dev: false,
            api: true,
            query: None,
        };
        assert_eq!(build_url(&args), API_DOCS_URL);
    }

    #[test]
    fn test_api_takes_precedence_over_dev() {
        let args = DocsArgs {
            dev: true,
            api: true,
            query: None,
        };
        assert_eq!(build_url(&args), API_DOCS_URL);
    }

    #[test]
    fn test_search_user_docs() {
        let args = DocsArgs {
            dev: false,
            api: false,
            query: Some("color".to_string()),
        };
        let url = build_url(&args);
        assert!(url.contains("google.com/search"));
        assert!(url.contains("site:cgl.ucsf.edu/chimerax/docs/user"));
        assert!(url.contains("color"));
    }

    #[test]
    fn test_search_dev_docs() {
        let args = DocsArgs {
            dev: true,
            api: false,
            query: Some("bundle".to_string()),
        };
        let url = build_url(&args);
        assert!(url.contains("google.com/search"));
        assert!(url.contains("site:cgl.ucsf.edu/chimerax/docs/devel"));
        assert!(url.contains("bundle"));
    }

    #[test]
    fn test_search_query_encoding() {
        let args = DocsArgs {
            dev: false,
            api: false,
            query: Some("open file".to_string()),
        };
        let url = build_url(&args);
        assert!(url.contains("open%20file"));
    }
}
