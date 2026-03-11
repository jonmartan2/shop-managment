use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Item {
    id: u32,
    image: String,
    name: String,
    price: f64,
    quantity: u32,
    sold: u32,
}

#[derive(Serialize, Deserialize)]
struct StoredData {
    next_id: u32,
    items: Vec<Item>,
}

const FILE_PATH: &str = "items.bin";

fn load_data() -> StoredData {
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
    }
}

fn save_data(data: &StoredData) {
    if let Ok(mut file) = File::create(FILE_PATH) {
        if let Ok(serialized) = serialize(data) {
            let _ = file.write_all(&serialized);
        }
    }
}

fn prompt(label: &str) -> String {
    print!("{}: ", label);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_with_default(label: &str, default: &str) -> String {
    print!("{} [{}]: ", label, default);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed
    }
}

fn list_items(data: &StoredData) {
    if data.items.is_empty() {
        println!("\n  (no items found)\n");
        return;
    }
    println!();
    println!(
        "  {:<5} {:<25} {:>10} {:>10} {:>8}",
        "ID", "Name", "Price", "Qty", "Sold"
    );
    println!("  {}", "-".repeat(62));
    for item in &data.items {
        println!(
            "  {:<5} {:<25} {:>10} {:>10} {:>8}",
            item.id,
            item.name,
            format!("${:.2}", item.price),
            item.quantity,
            item.sold
        );
    }
    println!();
}

fn add_item(data: &mut StoredData) {
    println!("\n── Add New Item ──");
    let name = prompt("  Name");
    if name.is_empty() {
        println!("  Cancelled.");
        return;
    }

    let price: f64 = loop {
        let s = prompt("  Price");
        match s.parse() {
            Ok(v) if v >= 0.0 => break v,
            _ => println!("  Invalid price, try again."),
        }
    };

    let quantity: u32 = loop {
        let s = prompt("  Quantity");
        match s.parse() {
            Ok(v) => break v,
            _ => println!("  Invalid quantity, try again."),
        }
    };

    let image = prompt("  Image URL or path (leave blank for none)");
    let image = if image.is_empty() {
        "https://via.placeholder.com/300x200?text=No+Image".to_string()
    } else {
        image
    };

    let id = data.next_id;
    data.next_id += 1;
    data.items.push(Item {
        id,
        image,
        name: name.clone(),
        price,
        quantity,
        sold: 0,
    });
    save_data(data);
    println!("  ✅ Added \"{}\" with ID {}.\n", name, id);
}

fn edit_item(data: &mut StoredData) {
    println!("\n── Edit Item ──");
    list_items(data);
    if data.items.is_empty() {
        return;
    }

    let id_str = prompt("  Enter item ID to edit");
    let id: u32 = match id_str.parse() {
        Ok(v) => v,
        Err(_) => {
            println!("  Invalid ID.");
            return;
        }
    };

    let item = match data.items.iter_mut().find(|i| i.id == id) {
        Some(i) => i,
        None => {
            println!("  Item with ID {} not found.", id);
            return;
        }
    };

    println!("  Editing \"{}\". Press Enter to keep current value.\n", item.name);

    let name = prompt_with_default("  Name", &item.name);

    let price: f64 = loop {
        let s = prompt_with_default("  Price", &format!("{:.2}", item.price));
        match s.parse() {
            Ok(v) if v >= 0.0 => break v,
            _ => println!("  Invalid price, try again."),
        }
    };

    let quantity: u32 = loop {
        let s = prompt_with_default("  Quantity", &item.quantity.to_string());
        match s.parse() {
            Ok(v) => break v,
            _ => println!("  Invalid quantity, try again."),
        }
    };

    let sold: u32 = loop {
        let s = prompt_with_default("  Sold", &item.sold.to_string());
        match s.parse() {
            Ok(v) => break v,
            _ => println!("  Invalid value, try again."),
        }
    };

    let image = prompt_with_default("  Image URL or path", &item.image);

    item.name = name.clone();
    item.price = price;
    item.quantity = quantity;
    item.sold = sold;
    item.image = image;

    save_data(data);
    println!("  ✅ Item {} \"{}\" updated.\n", id, name);
}

fn delete_item(data: &mut StoredData) {
    println!("\n── Delete Item ──");
    list_items(data);
    if data.items.is_empty() {
        return;
    }

    let id_str = prompt("  Enter item ID to delete");
    let id: u32 = match id_str.parse() {
        Ok(v) => v,
        Err(_) => {
            println!("  Invalid ID.");
            return;
        }
    };

    let pos = data.items.iter().position(|i| i.id == id);
    match pos {
        Some(idx) => {
            let name = data.items[idx].name.clone();
            let confirm = prompt(&format!("  Delete \"{}\"? (y/N)", name));
            if confirm.to_lowercase() == "y" {
                data.items.remove(idx);
                save_data(data);
                println!("  ✅ Deleted \"{}\".\n", name);
            } else {
                println!("  Cancelled.\n");
            }
        }
        None => println!("  Item with ID {} not found.\n", id),
    }
}

fn main() {
    println!("╔══════════════════════════════╗");
    println!("║     Shop CLI — items.bin     ║");
    println!("╚══════════════════════════════╝");

    loop {
        println!("  [1] List items");
        println!("  [2] Add item");
        println!("  [3] Edit item");
        println!("  [4] Delete item");
        println!("  [q] Quit");
        let choice = prompt("\nChoice");

        let mut data = load_data();

        match choice.as_str() {
            "1" => list_items(&data),
            "2" => add_item(&mut data),
            "3" => edit_item(&mut data),
            "4" => delete_item(&mut data),
            "q" | "Q" => {
                println!("  Bye!");
                break;
            }
            _ => println!("  Unknown option.\n"),
        }
    }
}
