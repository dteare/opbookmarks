/// Uses the `op` CLI to load Account, Vault, and Item information
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AccountOverview {
    pub email: String,
    pub url: String,
    pub user_uuid: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AccountDetails {
    pub id: String,
    pub name: String,
    pub domain: String,

    #[serde(rename = "type")]
    pub account_type: String,
    pub state: String,
    pub created_at: String,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct VaultOverview {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct VaultDetails {
    pub id: String,
    pub name: String,

    pub attribute_version: usize,
    pub content_version: usize,

    #[serde(rename = "type")]
    pub vault_type: String,

    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ItemOverview {
    pub id: String,
    pub title: String,
    pub version: usize,
    pub vault: VaultOverview,
    pub category: String,
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ItemDetails {
    pub id: String,
    pub title: String,
    pub tags: Option<Vec<String>>,
    pub version: usize,
    pub vault: VaultOverview,
    pub category: String,
    pub last_edited_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub urls: Option<Vec<OPURL>>,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct OPURL {
    pub primary: Option<bool>,
    pub href: String,
}

#[derive(Debug)]
pub enum Error {
    OPCLI(String),
    Deserialize(serde_json::Error),
    // Serialize(serde_json::Error),
}

pub fn load_all_accounts(account_user_uuids: &Vec<String>) -> Result<Vec<AccountDetails>, Error> {
    let accounts = find_accounts(account_user_uuids);

    match accounts {
        Ok(accounts) => {
            let mut details: Vec<AccountDetails> = vec![];
            for account in accounts.iter() {
                let ad = get_account(&account.user_uuid);

                match ad {
                    Ok(ad) => details.push(ad),
                    Err(e) => {
                        eprint!("Error loading account details: {:?}", e);
                        return Err(Error::OPCLI(format!(
                            "Failed to load details for account {}",
                            account.user_uuid
                        )));
                    }
                }
            }

            Ok(details)
        }
        Err(e) => Err(e),
    }
}

pub fn find_accounts(account_user_uuids: &Vec<String>) -> Result<Vec<AccountOverview>, Error> {
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

    let accounts: Result<Vec<AccountOverview>, Error> =
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
                let mut specified_accounts: Vec<AccountOverview> = vec![];
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

// op --account BXRGOJ2Z5JB4RMA7FUYUURELUE --format json account get
pub fn get_account(user_id: &String) -> Result<AccountDetails, Error> {
    let output = Command::new("op")
        .arg("--account")
        .arg(user_id)
        .arg("--format")
        .arg("json")
        .arg("account")
        .arg("get")
        .output()
        .expect("failed to execute `op` command for get_account");
    let json = output.stdout;
    let error = output.stderr;

    if error.len() > 0 {
        return Err(Error::OPCLI(
            std::str::from_utf8(error.as_slice()).unwrap().to_string(),
        ));
    }

    serde_json::from_slice(json.as_slice()).map_err(|e| Error::Deserialize(e))
}

pub fn load_all_vaults(account_id: &String) -> Result<Vec<VaultDetails>, Error> {
    let vaults = find_vaults(&account_id);

    match vaults {
        Ok(vaults) => {
            let mut details: Vec<VaultDetails> = vec![];
            for vault in vaults.iter() {
                let ad = get_vault(&account_id, &vault.id);

                match ad {
                    Ok(ad) => details.push(ad),
                    Err(e) => {
                        eprint!(
                            "Error loading vault details for account {}: {:?}",
                            account_id, e
                        );
                        return Err(Error::OPCLI(format!(
                            "Failed to load details for account {}",
                            account_id
                        )));
                    }
                }
            }

            Ok(details)
        }
        Err(e) => Err(e),
    }
}

pub fn find_vaults(account_id: &String) -> Result<Vec<VaultOverview>, Error> {
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("--account")
        .arg(account_id)
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

// op --account BXRGOJ2Z5JB4RMA7FUYUURELUE --format json vault get jnnjfdrzr5rawkimmsvp3zzzxe
pub fn get_vault(account_id: &String, vault_id: &String) -> Result<VaultDetails, Error> {
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("--account")
        .arg(account_id)
        .arg("vault")
        .arg("get")
        .arg(vault_id)
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

pub fn load_all_items(account_id: &String, vault_id: &String) -> Result<Vec<ItemDetails>, Error> {
    let items = find_items(&account_id, &vault_id);

    match items {
        Ok(items) => {
            let mut details: Vec<ItemDetails> = vec![];
            for item in items.iter() {
                let item_details = get_item(&account_id, &vault_id, &item.id);

                match item_details {
                    Ok(d) => details.push(d),
                    Err(e) => {
                        eprint!(
                            "Error loading item {} in vault {} for account {}: {:?}",
                            item.id, vault_id, account_id, e
                        );
                        return Err(Error::OPCLI(format!(
                            "Failed to load details for account {}",
                            account_id
                        )));
                    }
                }
            }

            Ok(details)
        }
        Err(e) => Err(e),
    }
}

pub fn find_items(account_id: &String, vault_id: &String) -> Result<Vec<ItemOverview>, Error> {
    let output = Command::new("op")
        .arg("--format")
        .arg("json")
        .arg("--account")
        .arg(account_id)
        .arg("item")
        .arg("list")
        .arg("--vault")
        .arg(vault_id)
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

// op --account BXRGOJ2Z5JB4RMA7FUYUURELUE --vault jnnjfdrzr5rawkimmsvp3zzzxe --format json item get fu5rgmahfihx4j6lludeyx3oei
pub fn get_item(
    account_id: &String,
    vault_id: &String,
    item_id: &String,
) -> Result<ItemDetails, Error> {
    let output = Command::new("op")
        .arg("--account")
        .arg(account_id)
        .arg("--vault")
        .arg(vault_id)
        .arg("--format")
        .arg("json")
        .arg("item")
        .arg("get")
        .arg(item_id)
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
