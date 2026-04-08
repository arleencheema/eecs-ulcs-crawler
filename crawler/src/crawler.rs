use std::collections::HashMap;

use common::{AppError, Document};
use regex::Regex;
use scraper::{Html, Selector};

const BULLETIN_URL: &str = "https://bulletin.engin.umich.edu/courses/eecs/";

// ── Network ───────────────────────────────────────────────────────────────────

pub async fn fetch_bulletin_html() -> Result<String, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                     (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| AppError::CrawlError(format!("client build: {e}")))?;

    let html = client
        .get(BULLETIN_URL)
        .send()
        .await
        .map_err(|e| AppError::CrawlError(format!("GET {BULLETIN_URL}: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::CrawlError(format!("reading body: {e}")))?;

    Ok(html)
}

// ── Core parser ───────────────────────────────────────────────────────────────

pub fn parse_bulletin(html: &str) -> HashMap<u32, Document> {
    let doc = Html::parse_document(html);
    let p_sel = Selector::parse("p").unwrap();
    let strong_sel = Selector::parse("strong").unwrap();
    let course_re = Regex::new(r"EECS\s+(\d{3})\.").unwrap();
    let credits_re = Regex::new(r"\((\d+(?:-\d+)?)\s+credits?\)").unwrap();
    let mut map = HashMap::new();

    for p in doc.select(&p_sel) {
        // Get strong text — skip if no strong child
        let strong_text = match p.select(&strong_sel).next() {
            Some(s) => s.text().collect::<String>().replace('\u{a0}', " "),
            None => continue,
        };

        // Must match EECS NNN.
        let caps = match course_re.captures(&strong_text) {
            Some(c) => c,
            None => continue,
        };
        let number: u32 = caps[1].parse().unwrap_or(0);
        if number < 300 || number > 499 { continue; }

        // Full <p> text for description extraction
        let full_text = p.text().collect::<String>().replace('\u{a0}', " ");

        // Credits from full text
        let credits = credits_re.captures(&full_text)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| "3".to_string());

        // Title: from strong text, between first ". " and second ". "
        let title = {
            let after_num = strong_text
                .splitn(2, ". ")
                .nth(1)
                .unwrap_or("")
                .trim();
            // Strip cross-listing like " (ROB 380)" and trailing dot
            let t = after_num.split(" (").next().unwrap_or(after_num);
            t.trim_end_matches('.').trim().to_string()
        };

        // Description: everything after "(N credits)" or "(N credit)",
        // trimmed, cut before "CourseProfile"
        let description = credits_re.find(&full_text)
            .map(|m| {
                let after = &full_text[m.end()..];
                let cut = after.find("CourseProfile").unwrap_or(after.len());
                after[..cut].trim().to_string()
            })
            .unwrap_or_default();

        if title.is_empty() { continue; }

        map.insert(number, Document {
            id: number,
            title,
            description,
            credits,
            level: if number >= 400 { 400 } else { 300 },
            url: format!("https://bulletin.engin.umich.edu/courses/eecs/#eecs-{number}"),
            workload_score: 0.0,
        });
    }

    println!("[parse] found {} courses (EECS 300-499)", map.len());
    map
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Fetch and parse every EECS 300–499 course from the Engineering Bulletin.
/// Returns `(raw_html, docs)` so the caller can inspect the HTML when parsing
/// yields nothing.
pub async fn crawl_all() -> Result<(String, Vec<Document>), AppError> {
    let html = fetch_bulletin_html().await?;
    let mut docs: Vec<Document> = parse_bulletin(&html).into_values().collect();
    docs.sort_by_key(|d| d.id);
    Ok((html, docs))
}
