extern crate notify;

mod op;

use op::{find_accounts, find_items, find_vaults, Account, Item, Vault};

use clap::Parser;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{collections::HashMap, process::exit};

#[derive(Parser)]
struct Cli {
    /// The path to export the metadata files to. To use the same path that 1Password 7 used, specify ~/Library/Containers/com.agilebits.onepassword7/Data/Library/Caches/Metadata/1Password
    #[clap(parse(from_os_str))]
    export_path: std::path::PathBuf,

    /// The path to the 1Password 8 database file to watch. Typically ~/Library/Group\ Containers/2BUA8C4S2C.com.1password/Library/Application\ Support/1Password/Data
    #[clap(parse(from_os_str), short, long)]
    watch_path: Option<std::path::PathBuf>,

    /// Account user UUIDs to generate metadata for. Leave empty to export bookmarks for all accounts. Use spaces to separate multiple accounts. UUIDs can be found using `op account list`.
    accounts: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct OP7ItemMetaData {
    uuid: String,

    #[serde(rename = "profileUUID")]
    profile_uuid: String,

    #[serde(rename = "vaultUUID")]
    vault_uuid: String,

    #[serde(rename = "categoryUUID")]
    category_uuid: String,

    #[serde(rename = "itemTitle")]
    item_title: String,

    #[serde(rename = "itemDescription")]
    item_description: String,

    #[serde(rename = "websiteURLs")]
    website_urls: Vec<String>,

    #[serde(rename = "accountName")]
    account_name: String,

    #[serde(rename = "vaultName")]
    vault_name: String,

    #[serde(rename = "categoryPluralName")]
    category_plural_name: String,

    #[serde(rename = "categorySingularName")]
    category_singular_name: String,

    #[serde(rename = "modifiedAt")]
    modified_at: usize,

    #[serde(rename = "createdAt")]
    created_at: usize,
}

fn main() {
    let args = Cli::parse();
    if args.accounts.len() == 0 {
        println!("Generating metadata for all accounts...");
    } else {
        println!("Generating metadata for {:?}", args.accounts);
    }

    generate_opbookmarks(&args.accounts, &args.export_path);

    // Watch for changes
    if let Some(path) = args.watch_path {
        println!("Watching 1Password 8 data folder for changes ({:?})", path);
        if let Err(e) = watch(path, &args.accounts, &args.export_path) {
            println!("error: {:?}", e)
        }
    }
}

fn generate_opbookmarks(account_user_uuids: &Vec<String>, export_path: &std::path::PathBuf) {
    let accounts = find_accounts(account_user_uuids);

    if let Err(err) = accounts {
        eprintln!("Failed to load accounts: {:?}", err);
        exit(1);
    }

    let accounts = accounts.unwrap();
    let mut vaults_by_account: HashMap<Account, Vec<Vault>> = HashMap::new();
    let mut items_by_vault: HashMap<Vault, Vec<Item>> = HashMap::new();

    println!(
        "Exporting bookmarks for accounts {:?}",
        accounts
            .iter()
            .map(|a| a.user_uuid.clone())
            .collect::<Vec<String>>()
    );

    // Collect the vaults for each account
    for account in accounts.iter() {
        let vaults = find_vaults(account);

        match vaults {
            Ok(vaults) => {
                vaults_by_account.insert((*account).clone(), vaults);
            }
            Err(err) => {
                eprintln!(
                    "Failed to load vaults for account {}: {:?}",
                    account.user_uuid, err
                );
            }
        }
    }

    // Collect the items for each vault
    for (account, vaults) in vaults_by_account.iter() {
        for vault in vaults.iter() {
            let items = find_items(account, vault);

            match items {
                Ok(items) => {
                    items_by_vault.insert((*vault).clone(), items);
                }
                Err(err) => {
                    eprintln!(
                        "Failed to load items for vault {} in account {}: {:?}",
                        vault.id, account.user_uuid, err
                    )
                }
            }
        }
    }

    // Write out metadata for each item
    for (account, vaults) in vaults_by_account.iter() {
        for vault in vaults.iter() {
            let items = items_by_vault.get(vault);

            match items {
                Some(items) => {
                    write_items(export_path, items, vault, account);
                }
                None => {
                    eprint!("Unexpected None for items in vault {}", vault.id);
                }
            }
        }
    }
    println!("Metadata files created.");
}

fn write_items(
    export_path: &std::path::PathBuf,
    items: &Vec<Item>,
    vault: &Vault,
    account: &Account,
) {
    let mut path = export_path.clone();
    path.push(account.user_uuid.clone());

    for item in items.iter() {
        let op7_item = create_op7_metadata(&item, &vault, &account);

        match serde_json::to_string(&op7_item) {
            Ok(json) => {
                let mut path = path.clone();
                path.push(format!(
                    "{}_{}.onepassword-item-metadata",
                    vault.id, item.id
                ));
                write_file(path, json);
            }
            Err(err) => {
                eprint!(
                    "Error serializing item json for vault {}: {}",
                    vault.id, err
                );
            }
        };
    }
}

fn create_op7_metadata(item: &Item, vault: &Vault, account: &Account) -> OP7ItemMetaData {
    return OP7ItemMetaData {
        uuid: item.id.clone(),
        item_description: format!("Login from {}", &vault.name.clone().unwrap()),
        item_title: item.title.clone(),
        vault_name: vault.name.clone().unwrap().clone(),
        vault_uuid: vault.id.clone(),
        category_plural_name: item.category.clone(), // TODO: Map SECURE_NOTE, etc
        profile_uuid: account.user_uuid.clone(),
        website_urls: vec![],
        category_singular_name: item.category.clone(),
        category_uuid: "001".to_string(),
        account_name: "".to_string(), // TODO: Not sure anyone uses this?
        modified_at: 0,               // TODO: parse item.modified_at
        created_at: 0,                // TODO: parse item.created_at,
    };
}

fn write_file(path: std::path::PathBuf, contents: String) {
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    let path = Path::new(&path);
    let display = path.display();

    let folder = path.parent().unwrap();
    std::fs::create_dir_all(folder).unwrap();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(contents.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn watch(
    path: std::path::PathBuf,
    account_user_uuids: &Vec<String>,
    export_path: &std::path::PathBuf,
) -> notify::Result<()> {
    use notify::DebouncedEvent;
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(5))?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::NoticeRemove(path) => {
                    // SQLite removes the journal file after merging the contents with 1password.sqlite
                    if path.ends_with("1password.sqlite-journal") {
                        println!("1Password 8 data file changed. Updating metadata files...");
                        generate_opbookmarks(account_user_uuids, export_path);
                    } else {
                        println!("Ignoring NoticeRemove of {:?}", path);
                    }
                }
                _ => {
                    println!("Ignoring event {:?}", event)
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
