use anyhow::Result;
use common::Document;
use indexer::Index;

#[tokio::main]
async fn main() -> Result<()> {
    println!("UMich CS Indexer starting…");

    let mut idx = Index::new();

    idx.add_document(Document {
        id: 1,
        title: "Web Systems".into(),
        description: "Large scale web systems search engines distributed computing".into(),
        credits: "4".into(),
        level: 400,
        url: "https://atlas.ai.umich.edu/course/EECS%20485/".into(),
        workload_score: 0.0,
    })?;

    idx.add_document(Document {
        id: 2,
        title: "Mobile App Development for Entrepreneurs".into(),
        description: "Build iOS Android apps project based easy low workload practical".into(),
        credits: "4".into(),
        level: 400,
        url: "https://atlas.ai.umich.edu/course/EECS%20441/".into(),
        workload_score: 0.0,
    })?;

    let query = "easy low workload";
    println!("Query: {query:?}\n");

    for (doc, score) in idx.search(query, None) {
        println!("  [{score:.3}] ({}) {}", doc.id, doc.title);
    }

    Ok(())
}
