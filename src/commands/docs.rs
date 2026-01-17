//! `echidna docs` command implementation.
//!
//! Opens ChimeraX documentation in the browser.

use crate::error::Result;

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
    // TODO: Implement docs command
    // - Open ChimeraX user guide by default
    // - --dev opens developer documentation
    // - --api opens API reference
    // - --search opens search with query

    let url = if args.dev {
        "https://www.cgl.ucsf.edu/chimerax/docs/devel/"
    } else if args.api {
        "https://www.cgl.ucsf.edu/chimerax/docs/devel/modules/"
    } else if let Some(ref query) = args.query {
        // TODO: Construct search URL
        println!("Search for: {}", query);
        return Ok(());
    } else {
        "https://www.cgl.ucsf.edu/chimerax/docs/user/"
    };

    println!("Opening: {}", url);

    // TODO: Actually open the URL in browser
    // open::that(url)?;

    Ok(())
}
