use uuid::Uuid;

pub fn make_slug(text: &str) -> String {
    slug::slugify(text)
}

pub fn unique_slug(text: &str, id: &Uuid) -> String {
    format!("{}-{}", slug::slugify(text), &id.to_string()[..8])
}