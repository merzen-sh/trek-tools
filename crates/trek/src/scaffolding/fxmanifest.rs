use askama::Template;

#[derive(Template)]
#[template(path = "../templates/fxmanifest.txt")]
pub struct Fxmanifest<'a> {
    pub description: &'a str,
    pub include_ui: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fxmanifest_with_ui() {
        let manifest = Fxmanifest {
            description: "Test resource with UI",
            include_ui: true,
        };
        let rendered = manifest.render().expect("Failed to render fxmanifest");
        assert!(rendered.contains("description 'Test resource with UI'"));
        assert!(rendered.contains("files {"));
        assert!(rendered.contains("'ui/dist/**'"));
    }

    #[test]
    fn test_fxmanifest_without_ui() {
        let manifest = Fxmanifest {
            description: "Simple standalone resource",
            include_ui: false,
        };
        let rendered = manifest.render().expect("Failed to render fxmanifest");
        assert!(rendered.contains("description 'Simple standalone resource'"));
        assert!(!rendered.contains("files {"));
        assert!(!rendered.contains("'ui/**'"));
    }
}
