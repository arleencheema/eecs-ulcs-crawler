use common::Document;

pub async fn crawl_all() -> Vec<Document> {
    let data = include_str!("../../data/courses.json");
    let docs: Vec<Document> = serde_json::from_str(data)
        .expect("Failed to parse data/courses.json");
    println!("Loaded {} courses from static data", docs.len());
    docs
}
