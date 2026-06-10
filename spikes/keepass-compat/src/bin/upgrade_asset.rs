use anyhow::{anyhow, Context, Result};
use keepass::{config::DatabaseVersion, db::Database, DatabaseKey};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("usage: upgrade_asset <input.kdbx> <output.kdbx>"))?;
    let output = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("usage: upgrade_asset <input.kdbx> <output.kdbx>"))?;
    if args.next().is_some() {
        return Err(anyhow!("usage: upgrade_asset <input.kdbx> <output.kdbx>"));
    }

    let tmp = temp_path_for(&output);

    if !input.exists() {
        return Err(anyhow!("input not found: {}", input.display()));
    }
    if canonical_for_compare(&input)? == canonical_for_compare(&output)? {
        return Err(anyhow!("input and output resolve to the same file"));
    }
    if output.exists() {
        return Err(anyhow!(
            "output already exists; remove it first if you want to regenerate: {}",
            output.display()
        ));
    }
    if tmp.exists() {
        return Err(anyhow!(
            "temporary output already exists: {}",
            tmp.display()
        ));
    }

    println!("Input : {}", input.display());
    println!("Output: {}", output.display());
    println!("Original file will not be modified.");
    let password = rpassword::prompt_password("KDBX master password: ")?;
    if password.is_empty() {
        return Err(anyhow!("empty password is not accepted by this spike"));
    }

    let mut f = File::open(&input).with_context(|| format!("open {}", input.display()))?;
    let mut db = Database::open(&mut f, DatabaseKey::new().with_password(&password))
        .map_err(|e| anyhow!("failed to open input database: {e:?}"))?;

    println!("Opened input version: {:?}", db.config.version);
    let entries_before = db.iter_all_entries().count();
    let groups_before = count_groups(&db);
    println!("Input counts: groups={groups_before}, entries={entries_before}");

    db.config.version = DatabaseVersion::KDB4(1);

    let result = (|| -> Result<()> {
        {
            let mut out =
                File::create(&tmp).with_context(|| format!("create tmp {}", tmp.display()))?;
            db.save(&mut out, DatabaseKey::new().with_password(&password))
                .map_err(|e| anyhow!("failed to save upgraded database: {e:?}"))?;
            out.flush()?;
            out.sync_all()?;
        }

        let mut verify_f = File::open(&tmp)?;
        let db2 = Database::open(&mut verify_f, DatabaseKey::new().with_password(&password))
            .map_err(|e| anyhow!("saved file verification failed: {e:?}"))?;
        let entries_after = db2.iter_all_entries().count();
        let groups_after = count_groups(&db2);
        println!("Verified output version: {:?}", db2.config.version);
        println!("Output counts: groups={groups_after}, entries={entries_after}");

        if entries_before != entries_after || groups_before != groups_after {
            return Err(anyhow!(
                "count mismatch after upgrade: groups {groups_before}->{groups_after}, entries {entries_before}->{entries_after}"
            ));
        }

        std::fs::rename(&tmp, &output)
            .with_context(|| format!("rename tmp to {}", output.display()))?;
        Ok(())
    })();

    if let Err(err) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }

    println!("Upgrade complete: {}", output.display());
    println!("Next: open this test file in Strongbox and verify entries before using it.");
    Ok(())
}

fn count_groups(db: &Database) -> usize {
    fn walk(g: keepass::db::GroupRef<'_>) -> usize {
        1 + g.groups().map(walk).sum::<usize>()
    }
    walk(db.root())
}

fn temp_path_for(output: &Path) -> PathBuf {
    let mut tmp = output.to_path_buf();
    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("kdbx");
    tmp.set_extension(format!("{ext}.tmp"));
    tmp
}

fn canonical_for_compare(path: &Path) -> std::io::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize();
    }
    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
    })?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(parent.canonicalize()?.join(file_name))
}
