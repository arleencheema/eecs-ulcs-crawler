#[tokio::main]
async fn main() {
    let docs = crawler::crawl_all().await;
    println!("── {} courses ──", docs.len());
    for d in &docs {
        println!("  EECS {}  |  {}  |  {} cr", d.id, d.title, d.credits);
    }
}
