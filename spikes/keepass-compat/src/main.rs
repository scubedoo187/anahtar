use anyhow::{anyhow, Context, Result};
use keepass::{
    config::DatabaseVersion,
    db::{fields, Database, GroupRef},
    DatabaseKey,
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

const PASSWORD: &str = "demopass";

#[derive(Debug)]
struct Fixture {
    file: &'static str,
    password: &'static str,
    expect_save: bool,
}

#[derive(Debug, Clone, Copy)]
struct Counts {
    groups: usize,
    entries: usize,
}

fn main() -> Result<()> {
    let fixture_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures".to_string());
    let fixture_dir = PathBuf::from(fixture_dir);
    let out_dir = PathBuf::from("out");
    std::fs::create_dir_all(&out_dir)?;

    let fixtures = [
        Fixture {
            file: "test_db_with_password.kdbx",
            password: PASSWORD,
            expect_save: false,
        },
        Fixture {
            file: "test_db_kdbx4_with_password_aes.kdbx",
            password: PASSWORD,
            expect_save: true,
        },
        // These are KDBX4.0 fixtures. keepass 0.13.8 can read them, but native save only
        // supports KDBX4.1. The spike also tests explicit upgrade-to-4.1 save below.
        Fixture {
            file: "test_db_kdbx4_with_password_argon2.kdbx",
            password: PASSWORD,
            expect_save: false,
        },
        Fixture {
            file: "test_db_kdbx4_with_password_argon2_chacha20.kdbx",
            password: PASSWORD,
            expect_save: false,
        },
        Fixture {
            file: "test_db_kdbx4_with_password_argon2id.kdbx",
            password: PASSWORD,
            expect_save: false,
        },
        Fixture {
            file: "test_db_kdbx4_with_totp_entry.kdbx",
            password: "test",
            expect_save: false,
        },
        Fixture {
            file: "test_db_kdbx41_features.kdbx",
            password: PASSWORD,
            expect_save: true,
        },
    ];

    let mut passed = 0usize;
    for fixture in fixtures {
        println!("\n=== {} ===", fixture.file);
        run_fixture(&fixture_dir, &out_dir, &fixture)
            .with_context(|| format!("fixture {} failed", fixture.file))?;
        passed += 1;
    }

    println!("\nSUMMARY: {passed} fixtures passed basic open/save/reopen checks");
    Ok(())
}

fn key(password: &str) -> DatabaseKey {
    DatabaseKey::new().with_password(password)
}

fn open_db(path: &Path, password: &str) -> Result<Database> {
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    Database::open(&mut f, key(password)).map_err(|e| anyhow!("Database::open: {e:?}"))
}

fn run_fixture(fixture_dir: &Path, out_dir: &Path, fixture: &Fixture) -> Result<()> {
    let path = fixture_dir.join(fixture.file);
    let mut db = open_db(&path, fixture.password)?;
    let before = count_db(&db);
    println!(
        "opened: version={:?}, root='{}', groups={}, entries={}",
        db.config.version,
        db.root().name,
        before.groups,
        before.entries
    );

    let titles: Vec<String> = db
        .iter_all_entries()
        .take(5)
        .filter_map(|e| e.get_title().map(|s| s.to_string()))
        .collect();
    println!("sample titles: {:?}", titles);

    let mut roundtrip_path = out_dir.join(fixture.file);
    roundtrip_path.set_extension("roundtrip.kdbx");

    let save_result = {
        let mut out = File::create(&roundtrip_path)?;
        db.save(&mut out, key(fixture.password))
    };

    match (save_result, fixture.expect_save) {
        (Ok(()), true) => {
            println!("save roundtrip: ok -> {}", roundtrip_path.display());
            let db2 = open_db(&roundtrip_path, fixture.password)?;
            let after = count_db(&db2);
            println!(
                "reopened roundtrip: version={:?}, groups={}, entries={}",
                db2.config.version, after.groups, after.entries
            );
            if before.entries != after.entries || before.groups != after.groups {
                return Err(anyhow!(
                    "roundtrip count changed: before={before:?}, after={after:?}"
                ));
            }
            if fixture.file == "test_db_kdbx41_features.kdbx" {
                check_kdbx41_writer_xml(&roundtrip_path, fixture.password)?;
            }
        }
        (Err(e), false) => {
            println!("save skipped/unsupported as expected: {e:?}");
        }
        (Ok(()), false) => {
            println!(
                "save unexpectedly succeeded for non-KDBX4 fixture -> {}",
                roundtrip_path.display()
            );
        }
        (Err(e), true) => return Err(anyhow!("expected save to work, got {e:?}")),
    }

    if matches!(db.config.version, DatabaseVersion::KDB4(1)) {
        mutate_save_reopen(db, out_dir, fixture, before, "mutated")?;
    } else {
        // Explicit conversion experiment: the writer only emits KDBX4.1, so test whether
        // an opened KDBX3/KDBX4.0 database can be upgraded by changing the config version.
        let original_version = db.config.version;
        db.config.version = DatabaseVersion::KDB4(1);
        match mutate_save_reopen(db, out_dir, fixture, before, "upgraded-mutated") {
            Ok(()) => println!("upgrade-to-KDBX4.1 save from {:?}: ok", original_version),
            Err(e) => println!(
                "upgrade-to-KDBX4.1 save from {:?}: failed: {e:?}",
                original_version
            ),
        }
    }

    Ok(())
}

fn mutate_save_reopen(
    mut db: Database,
    out_dir: &Path,
    fixture: &Fixture,
    before: Counts,
    suffix: &str,
) -> Result<()> {
    let mut root = db.root_mut();
    let marker_title = format!("Anahtar Spike Marker - {} - {suffix}", fixture.file);
    root.add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, &marker_title);
        e.set_unprotected(fields::USERNAME, "anahtar-spike-user");
        e.set_protected(fields::PASSWORD, "anahtar-spike-password");
        e.set_unprotected(fields::URL, "https://example.invalid/anahtar-spike");
        e.set_unprotected(
            fields::NOTES,
            "Created by keepass-compat-spike; safe synthetic data.",
        );
    });
    drop(root);

    let mut mutated_path = out_dir.join(fixture.file);
    mutated_path.set_extension(format!("{suffix}.kdbx"));
    let mut out = File::create(&mutated_path)?;
    db.save(&mut out, key(fixture.password))
        .map_err(|e| anyhow!("save {suffix}: {e:?}"))?;
    let db3 = open_db(&mutated_path, fixture.password)?;
    let found = db3
        .iter_all_entries()
        .any(|e| e.get_title() == Some(marker_title.as_str()));
    if !found {
        return Err(anyhow!("{suffix} marker entry not found after reopen"));
    }
    let after_mut = count_db(&db3);
    println!(
        "{suffix} save/reopen: ok -> {}, entries {} -> {}",
        mutated_path.display(),
        before.entries,
        after_mut.entries
    );
    Ok(())
}

fn count_db(db: &Database) -> Counts {
    fn walk(g: GroupRef<'_>) -> Counts {
        let mut c = Counts {
            groups: 1,
            entries: g.entries().count(),
        };
        for child in g.groups() {
            let sub = walk(child);
            c.groups += sub.groups;
            c.entries += sub.entries;
        }
        c
    }
    walk(db.root())
}

fn check_kdbx41_writer_xml(path: &Path, password: &str) -> Result<()> {
    let mut f = File::open(path)?;
    let xml = Database::get_xml(&mut f, key(password)).map_err(|e| anyhow!("get_xml: {e:?}"))?;
    let xml = String::from_utf8(xml)?;
    for needle in [
        "<EnableSearching>null</EnableSearching>",
        "<EnableAutoType>null</EnableAutoType>",
        "<DataTransferObfuscation>0</DataTransferObfuscation>",
    ] {
        if !xml.contains(needle) {
            return Err(anyhow!(
                "KDBX4.1 writer XML missing KeePassXC-compatible marker: {needle}"
            ));
        }
    }
    if xml.contains("<DataTransferObfuscation>False</DataTransferObfuscation>")
        || xml.contains("<DataTransferObfuscation>True</DataTransferObfuscation>")
    {
        return Err(anyhow!(
            "KDBX4.1 writer XML contains bool-string DataTransferObfuscation"
        ));
    }
    println!("KDBX4.1 writer XML checks: ok");
    Ok(())
}
