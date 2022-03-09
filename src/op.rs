/// Uses the `op` CLI to load Account, Vault, and Item information
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Account {
    pub email: String,
    pub url: String,
    pub user_uuid: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Vault {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub version: usize,
    pub vault: Vault,
    pub category: String,
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub enum Error {
    OPCLI(String),
    Deserialize(serde_json::Error),
    // Serialize(serde_json::Error),
}

pub fn find_accounts(account_user_uuids: &Vec<String>) -> Result<Vec<Account>, Error> {
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("account")
        .arg("list")
        .output()
        .expect("failed to execute `op` command");
    let json = output.stdout;
    let error = output.stderr;

    if error.len() > 0 {
        return Err(Error::OPCLI(
            std::str::from_utf8(error.as_slice()).unwrap().to_string(),
        ));
    }

    let accounts: Result<Vec<Account>, Error> =
        serde_json::from_slice(json.as_slice()).map_err(|e| Error::Deserialize(e));

    match accounts {
        Ok(accounts) => {
            if account_user_uuids.len() == 0 {
                println!(
                    "Including all found accounts for export: {}",
                    accounts.len()
                );
                Ok(accounts)
            } else {
                // Limit to the specified accounts
                let mut specified_accounts: Vec<Account> = vec![];
                for uuid in account_user_uuids.iter() {
                    match accounts.iter().find(|a| (*a).user_uuid == uuid.as_str()) {
                        Some(account) => {
                            specified_accounts.push(account.clone());
                        }
                        None => {
                            eprintln!(
                                "Cannot include specified account {} for export as it couldn't be found",
                                uuid
                            );
                        }
                    }
                }
                Ok(specified_accounts)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn find_vaults(account: &Account) -> Result<Vec<Vault>, Error> {
    println!("account={:?}", account);
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("--account")
        .arg(account.user_uuid.clone())
        .arg("vault")
        .arg("list")
        .output()
        .expect("failed to execute `op` command");
    let json = output.stdout;
    let error = output.stderr;

    if error.len() > 0 {
        return Err(Error::OPCLI(
            std::str::from_utf8(error.as_slice()).unwrap().to_string(),
        ));
    }

    serde_json::from_slice(json.as_slice()).map_err(|e| Error::Deserialize(e))
}

pub fn find_items(account: &Account, vault: &Vault) -> Result<Vec<Item>, Error> {
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
        return Err(Error::OPCLI(
            std::str::from_utf8(error.as_slice()).unwrap().to_string(),
        ));
    }

    serde_json::from_slice(json.as_slice()).map_err(|e| Error::Deserialize(e))
}
