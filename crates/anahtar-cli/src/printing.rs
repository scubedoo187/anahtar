use anahtar_core::{
    AuditReport, EntryDetail, EntrySummary, GroupSummary, TotpCode, UpgradeReport, VaultInfo,
    WriteReport,
};
use anyhow::Result;

pub fn print_vault_info(info: &VaultInfo, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(info)?);
    } else {
        println!("Path: {}", info.path.display());
        println!("Size: {} bytes", info.file_size_bytes);
        println!("Format: {}", info.version);
    }
    Ok(())
}

pub fn print_entries(entries: &[EntrySummary], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(entries)?);
    } else {
        for entry in entries {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                entry.id,
                entry.group_path,
                entry.title.as_deref().unwrap_or(""),
                entry.username.as_deref().unwrap_or(""),
                entry.url.as_deref().unwrap_or("")
            );
        }
    }
    Ok(())
}

pub fn print_dry_run(
    operation: &str,
    target: &std::path::Path,
    output: Option<&std::path::PathBuf>,
    json: bool,
) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::json!({
                "operation": operation,
                "dry_run": true,
                "target": target,
                "output": output,
                "would_write": false
            })
        );
    } else {
        println!("Operation: {operation}");
        println!("Target: {}", target.display());
        if let Some(output) = output {
            println!("Output: {}", output.display());
        }
        println!("Dry run: no file was written.");
    }
    Ok(())
}

pub fn print_groups(groups: &[GroupSummary], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(groups)?);
    } else {
        for group in groups {
            println!(
                "{}\t{}\tentries={}\tchildren={}",
                group.id, group.path, group.entry_count, group.child_group_count
            );
        }
    }
    Ok(())
}

pub fn print_audit(report: &AuditReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else if report.findings.is_empty() {
        println!("No audit findings.");
    } else {
        for finding in &report.findings {
            println!(
                "{}\t{}\t{}\t{}",
                finding.kind, finding.entry_id, finding.group_path, finding.message
            );
        }
    }
    Ok(())
}

pub fn print_write_report(report: &WriteReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!("Operation: {:?}", report.operation);
        println!("Input: {}", report.input_path.display());
        println!("Output: {}", report.output_path.display());
        println!("Input format: {}", report.input_version);
        println!("Output format: {}", report.output_version);
        println!(
            "Counts: groups {} -> {}, entries {} -> {}",
            report.input_group_count,
            report.output_group_count,
            report.input_entry_count,
            report.output_entry_count
        );
        if let Some(id) = &report.changed_entry_id {
            println!("Changed entry id: {id}");
        }
        if let Some(backup_path) = &report.backup_path {
            println!("Backup: {}", backup_path.display());
        }
        if let Some(final_target_path) = &report.final_target_path {
            println!("Final target: {}", final_target_path.display());
        }
        if report.final_target_path.is_some() {
            println!("Write complete. Target vault was safely updated in place.");
        } else {
            println!("Write complete. Original input was not modified.");
        }
        println!("Next: open the output file in Strongbox and manually verify it before using it as a primary vault.");
    }
    Ok(())
}

pub fn print_upgrade_report(report: &UpgradeReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        if let Some(warning) = &report.warning {
            println!("Warning: {warning}");
        }
        println!("Input: {}", report.input_path.display());
        println!("Output: {}", report.output_path.display());
        println!("Input format: {}", report.input_version);
        println!("Output format: {}", report.output_version);
        println!(
            "Input counts: groups={}, entries={}",
            report.input_group_count, report.input_entry_count
        );
        if report.dry_run {
            println!("Dry run: no file was written.");
        } else {
            println!(
                "Output counts: groups={}, entries={}",
                report.output_group_count.unwrap_or_default(),
                report.output_entry_count.unwrap_or_default()
            );
            println!("Upgrade complete. Original input was not modified.");
            println!("Next: open the output file in Strongbox and manually verify important entries before using it as a primary vault.");
        }
    }
    Ok(())
}

pub fn print_totp(code: &TotpCode) {
    println!("{}", code.code);
    println!("valid for {}s", code.valid_for_seconds);
}

pub fn print_detail(detail: &EntryDetail, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(detail)?);
    } else {
        println!("ID: {}", detail.id);
        println!("Group: {}", detail.group_path);
        println!("Title: {}", detail.title.as_deref().unwrap_or(""));
        println!("Username: {}", detail.username.as_deref().unwrap_or(""));
        println!("URL: {}", detail.url.as_deref().unwrap_or(""));
        println!("Notes: {}", detail.notes.as_deref().unwrap_or(""));
        match &detail.password {
            Some(password) => println!("Password: {password}"),
            None => println!("Password: <hidden; pass --reveal-password to display>"),
        }
        if !detail.custom_fields.is_empty() {
            println!("Custom fields:");
            for field in &detail.custom_fields {
                let marker = if field.protected {
                    "protected"
                } else {
                    "plain"
                };
                println!("  {} ({marker}): {}", field.key, field.value);
            }
        }
    }
    Ok(())
}
