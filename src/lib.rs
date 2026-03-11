use bincode::{deserialize, serialize};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u32,
    pub image: String,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
    pub sold: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sale {
    pub id: u32,
    pub item_id: u32,
    pub item_name: String,
    pub quantity: u32,
    pub price_at_sale: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredData {
    #[serde(default)]
    pub next_id: u32,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub sales: Vec<Sale>,
    #[serde(default)]
    pub next_sale_id: u32,
}

pub const FILE_PATH: &str = "items.bin";

pub fn load_data() -> StoredData {
    if let Ok(mut file) = File::open(FILE_PATH) {
        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).is_ok() {
            if let Ok(data) = deserialize(&buf) {
                return data;
            }
        }
    }
    StoredData {
        next_id: 0,
        items: Vec::new(),
        sales: Vec::new(),
        next_sale_id: 0,
    }
}

pub fn save_data(data: &StoredData) {
    if let Ok(mut file) = File::create(FILE_PATH) {
        if let Ok(serialized) = serialize(data) {
            let _ = file.write_all(&serialized);
        }
    }
}
