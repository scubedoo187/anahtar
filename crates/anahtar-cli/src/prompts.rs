use anahtar_core::EntryDetail;
use anyhow::Result;
use std::io::Write;

pub fn prompt_password() -> Result<String> {
    Ok(rpassword::prompt_password("KDBX master password: ")?)
}

pub fn prompt_entry_password_with_confirmation() -> Result<String> {
    let password = rpassword::prompt_password("Entry password: ")?;
    let confirm = rpassword::prompt_password("Confirm entry password: ")?;
    if password != confirm {
        anyhow::bail!("entry password confirmation did not match");
    }
    Ok(password)
}

pub fn confirm_group_delete(path: &str) -> Result<()> {
    println!("Delete group and all nested entries/groups?");
    println!("Group: {path}");
    print!("Type DELETE GROUP to confirm: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "DELETE GROUP" {
        anyhow::bail!("group delete confirmation failed");
    }
    Ok(())
}

pub fn confirm_delete(detail: &EntryDetail) -> Result<()> {
    println!("Delete entry?");
    println!("ID: {}", detail.id);
    println!("Title: {}", detail.title.as_deref().unwrap_or(""));
    println!("Username: {}", detail.username.as_deref().unwrap_or(""));
    println!("URL: {}", detail.url.as_deref().unwrap_or(""));
    print!("Type DELETE to confirm: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "DELETE" {
        anyhow::bail!("delete confirmation failed");
    }
    Ok(())
}
