use keepass::{
    db::{fields, Database},
    DatabaseKey,
};
use std::{fs::File, path::Path};

const PASSWORD: &str = "testpass";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = Path::new("test-vaults/generated/phase3-base.kdbx");
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir_all("test-vaults/generated/outputs")?;

    let mut db = Database::new();
    db.root_mut().edit(|root| root.name = "Root".to_string());

    let general_id = db
        .root_mut()
        .add_group()
        .edit(|g| g.name = "General".to_string())
        .id();

    let web_id = db
        .group_mut(general_id)
        .expect("General group exists")
        .add_group()
        .edit(|g| g.name = "Web".to_string())
        .id();

    let email_id = db
        .group_mut(general_id)
        .expect("General group exists")
        .add_group()
        .edit(|g| g.name = "Email".to_string())
        .id();

    db.group_mut(web_id)
        .expect("Web group exists")
        .add_entry()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "Github Test");
            e.set_unprotected(fields::USERNAME, "github-user");
            e.set_protected(fields::PASSWORD, "github-pass");
            e.set_unprotected(fields::URL, "https://github.com");
            e.set_unprotected(
                fields::NOTES,
                "Synthetic web entry for Anahtar Phase 3 tests",
            );
        });

    db.group_mut(email_id)
        .expect("Email group exists")
        .add_entry()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "Email Test");
            e.set_unprotected(fields::USERNAME, "email-user");
            e.set_protected(fields::PASSWORD, "email-pass");
            e.set_unprotected(fields::URL, "https://mail.example.com");
            e.set_unprotected(
                fields::NOTES,
                "Synthetic email entry for Anahtar Phase 3 tests",
            );
        });

    db.group_mut(web_id)
        .expect("Web group exists")
        .add_entry()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "Duplicate Title");
            e.set_unprotected(fields::USERNAME, "duplicate-web-user");
            e.set_protected(fields::PASSWORD, "duplicate-web-pass");
            e.set_unprotected(fields::URL, "https://web.example.com/duplicate");
        });

    db.group_mut(email_id)
        .expect("Email group exists")
        .add_entry()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "Duplicate Title");
            e.set_unprotected(fields::USERNAME, "duplicate-email-user");
            e.set_protected(fields::PASSWORD, "duplicate-email-pass");
            e.set_unprotected(fields::URL, "https://mail.example.com/duplicate");
        });

    let mut out = File::create(output)?;
    db.save(&mut out, DatabaseKey::new().with_password(PASSWORD))?;
    println!(
        "Generated {} with password `{}`",
        output.display(),
        PASSWORD
    );
    Ok(())
}
