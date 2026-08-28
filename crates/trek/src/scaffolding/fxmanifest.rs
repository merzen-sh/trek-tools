use askama::Template;

#[derive(Template)]
#[template(path = "../templates/fxmanifest.txt")]
pub struct Fxmanifest<'a> {
    pub description: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fxmanifest_renders_description() {
        let manifest = Fxmanifest {
            description: "Test resource",
        };
        let rendered = manifest.render().expect("Failed to render fxmanifest");
        assert!(rendered.contains("description(\"Test resource\")"));
        assert!(!rendered.contains("files {"));
    }
}
