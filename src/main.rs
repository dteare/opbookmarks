extern crate notify;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{
    collections::HashMap,
    process::{exit, Command},
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Account {
    email: String,
    url: String,
    user_uuid: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Vault {
    id: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Item {
    id: String,
    title: String,
    version: usize,
    vault: Vault,
    category: String,
    last_edited_by: String,
    created_at: String,
    updated_at: String,
}

fn main() {
    let accounts = find_accounts();

    if let Err(err) = accounts {
        println!("Failed to load accounts: {}", err);
        exit(1);
    }

    let accounts = accounts.unwrap();
    let mut vaults_by_account: HashMap<Account, Vec<Vault>> = HashMap::new();
    let mut items_by_vault: HashMap<Vault, Vec<Item>> = HashMap::new();

    for account in accounts.iter() {
        let vaults = find_vaults(account);

        match vaults {
            Ok(vaults) => {
                vaults_by_account.insert((*account).clone(), vaults);
            }
            Err(err) => {
                eprintln!(
                    "Failed to load vaults for account {}: {}",
                    account.user_uuid, err
                );
            }
        }
    }

    for (account, vaults) in vaults_by_account.iter() {
        if account.url == "agilebits.1password.com"
            || account.user_uuid == "45AID2SX2JBFHDA2IRNB2WXZLA"
            || account.user_uuid == "KDFJ5CQRGBDILKE5UKSPCUB46A"
            || account.user_uuid == "MIUAPSEL2FGPVEH6SI54XXCKEM"
        {
            // Skip my massive accounts during testing
            println!("Skipping account: {:?}", account);
            continue;
        }

        for vault in vaults.iter() {
            let items = find_items(account, vault);

            match items {
                Ok(items) => {
                    items_by_vault.insert((*vault).clone(), items);
                }
                Err(err) => {
                    eprintln!(
                        "Failed to load items for vault {} in account {}: {}",
                        vault.id, account.user_uuid, err
                    )
                }
            }
        }
    }

    println!("Found {} accounts", accounts.len());
    println!(
        "Found {} vaults in {} accounts",
        vaults_by_account.len(),
        accounts.len()
    );

    for (account, vaults) in vaults_by_account.iter() {
        for vault in vaults.iter() {
            let items = items_by_vault.get(vault);

            match items {
                Some(items) => {
                    write_items(items, vault, account);
                }
                None => {
                    eprint!("Unexpected None for items in vault {}", vault.id);
                }
            }
        }
    }
    println!("{:?}", items_by_vault);

    // Watch for changes
    if let Err(e) = watch() {
        println!("error: {:?}", e)
    }
}

fn write_items(items: &Vec<Item>, vault: &Vault, account: &Account) {
    match serde_json::to_string(&items) {
        Ok(json) => {
            println!(
                "Item json for vault {:?} in account {:?}:\n{}",
                vault, account, json
            );
            write_file(
                format!("dist/{}/{}/items.json", account.user_uuid, vault.id),
                json,
            );
        }
        Err(err) => {
            eprint!(
                "Error serializing item json for vault {}: {}",
                vault.id, err
            );
        }
    };
}

fn find_accounts() -> Result<Vec<Account>, serde_json::Error> {
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("account")
        .arg("list")
        .output()
        .expect("failed to execute `op` command");
    let json = output.stdout;

    let accounts: Vec<Account> = serde_json::from_slice(json.as_slice())?;

    Ok(accounts)
}

fn write_file(path: String, contents: String) {
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

fn find_vaults(account: &Account) -> Result<Vec<Vault>, serde_json::Error> {
    println!("account={:?}", account);
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("--account")
        .arg(account.url.clone())
        .arg("vault")
        .arg("list")
        .output()
        .expect("failed to execute `op` command");
    let json = output.stdout;
    let error = output.stderr;

    if error.len() > 0 {
        println!(
            "Error from op: {}",
            std::str::from_utf8(error.as_slice()).unwrap()
        );
    }

    let vaults: Vec<Vault> = serde_json::from_slice(json.as_slice())?;

    Ok(vaults)
}

fn find_items(account: &Account, vault: &Vault) -> Result<Vec<Item>, serde_json::Error> {
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("--account")
        .arg(account.url.clone())
        .arg("item")
        .arg("list")
        .arg("--vault")
        .arg(vault.id.clone())
        .output()
        .expect("failed to execute `op` command");
    let json = output.stdout;
    let error = output.stderr;

    if error.len() > 0 {
        println!(
            "Error from op item list: {}",
            std::str::from_utf8(error.as_slice()).unwrap()
        );
    }

    let items: Vec<Item> = serde_json::from_slice(json.as_slice())?;

    Ok(items)
}

fn watch() -> notify::Result<()> {
    use notify::DebouncedEvent;
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(5))?;

    watcher.watch("./src", RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(_) => {
                    println!("Event: {:?}", event)
                }
                _ => {}
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
