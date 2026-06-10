use anahtar_core::{list_entries, open_database};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: list_test_vault <kdbx>");
    let db = open_database(path, "testpass")?;
    for e in list_entries(&db) {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            e.id,
            e.group_path,
            e.title.unwrap_or_default(),
            e.username.unwrap_or_default(),
            e.url.unwrap_or_default()
        );
    }
    Ok(())
}
