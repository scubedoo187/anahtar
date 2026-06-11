use anahtar_core::{
    open_database_with_credentials, show_entry_by_selector, EntrySelector, VaultCredentials,
};
use anyhow::Result;
use std::path::PathBuf;

use crate::cli::EntrySelectorArgs;

pub fn selector_from_args(args: EntrySelectorArgs) -> Result<EntrySelector> {
    let selectors = [
        args.selector.map(EntrySelector::Auto),
        args.id.map(EntrySelector::Id),
        args.title.map(EntrySelector::Title),
        args.url.map(EntrySelector::Url),
        args.username.map(EntrySelector::Username),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    match selectors.as_slice() {
        [selector] => Ok(selector.clone()),
        [] => anyhow::bail!("entry selector required: provide a positional selector or one of --id/--title/--url/--username"),
        _ => anyhow::bail!("provide exactly one entry selector"),
    }
}

pub fn resolve_selector_id(
    vault: &PathBuf,
    credentials: &VaultCredentials,
    selector: &EntrySelector,
) -> Result<String> {
    let db = open_database_with_credentials(vault, credentials)?;
    Ok(show_entry_by_selector(&db, selector, false)?.id)
}
