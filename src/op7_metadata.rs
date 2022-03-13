/// Create metadata files that conform to the format used by 1Password 7
use crate::op::{AccountDetails, ItemDetails, VaultDetails};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct OP7ItemMetaData {
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

pub fn write_items(
    export_path: &std::path::PathBuf,
    items: &Vec<ItemDetails>,
    vault: &VaultDetails,
    account: &AccountDetails,
) {
    let mut path = export_path.clone();
    path.push(account.id.clone());

    for item in items.iter() {
        let op7_item = create_op7_metadata(&item, &vault, &account.id);

        match serde_json::to_string(&op7_item) {
            Ok(json) => {
                let mut path = path.clone();
                path.push(format!(
                    "{}_{}.onepassword-item-metadata",
                    vault.id, item.id
                ));
                crate::util::write_file(path, json);
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

fn create_op7_metadata(
    item: &ItemDetails,
    vault: &VaultDetails,
    account_id: &String,
) -> OP7ItemMetaData {
    let website_urls = match &item.urls {
        Some(urls) => {
            let mut result: Vec<String> = vec![];
            for url in urls.iter() {
                result.push(url.href.clone());
            }
            result
        }
        None => vec![],
    };

    return OP7ItemMetaData {
        uuid: item.id.clone(),
        item_description: format!("Login from {}", &vault.name.clone()),
        item_title: item.title.clone(),
        vault_name: vault.name.clone(),
        vault_uuid: vault.id.clone(),
        category_plural_name: item.category.clone(), // TODO: Map SECURE_NOTE, etc
        profile_uuid: account_id.clone(),
        website_urls: website_urls,
        category_singular_name: item.category.clone(),
        category_uuid: "001".to_string(),
        account_name: "".to_string(), // TODO: Not sure anyone uses this?
        modified_at: 0,               // TODO: parse item.modified_at
        created_at: 0,                // TODO: parse item.created_at,
    };
}
