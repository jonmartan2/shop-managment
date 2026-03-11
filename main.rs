use axum::{
    Router,
    extract::{Json as ExtractJson, Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
struct Item {
    id: u32,
    image: String,
    name: String,
    price: f64,
    quantity: u32,
    sold: u32,
}

#[derive(Deserialize)]
struct NewItem {
    image: String,
    name: String,
    price: f64,
    quantity: u32,
}

#[derive(Serialize, Deserialize)]
struct StoredData {
    next_id: u32,
    items: Vec<Item>,
}

#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<StoredData>>,
    file_path: String,
}

fn load_data(path: &str) -> StoredData {
    if let Ok(mut file) = File::open(path) {
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

fn save_data(path: &str, data: &StoredData) {
    if let Ok(mut file) = File::create(path) {
        if let Ok(serialized) = serialize(data) {
            let _ = file.write_all(&serialized);
        }
    }
}

#[tokio::main]
async fn main() {
    // Ensure uploads directory exists
    fs::create_dir_all("static/uploads").unwrap();

    let file_path = "items.bin".to_string();
    let mut stored_data = load_data(&file_path);

    let state = AppState {
        data: Arc::new(Mutex::new(stored_data)),
        file_path,
    };

    let app = Router::new()
        .route("/", get(serve_page))
        .route("/items", get(get_items).post(add_item))
        .route("/items/:id/sell", post(sell_item))
        .route("/upload", post(upload_image))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    println!("Running on http://0.0.0.0:5000");
    axum::serve(listener, app).await.unwrap();
}

async fn serve_page() -> Html<String> {
    Html(std::fs::read_to_string("static/index.html").unwrap())
}

async fn get_items(State(state): State<AppState>) -> impl IntoResponse {
    let data = state.data.lock().await;
    (StatusCode::OK, axum::Json(data.items.clone()))
}

async fn add_item(
    State(state): State<AppState>,
    ExtractJson(new_item): ExtractJson<NewItem>,
) -> impl IntoResponse {
    let mut data = state.data.lock().await;
    let id = data.next_id;
    data.next_id += 1;
    data.items.push(Item {
        id,
        image: new_item.image,
        name: new_item.name,
        price: new_item.price,
        quantity: new_item.quantity,
        sold: 0,
    });
    save_data(&state.file_path, &data);
    StatusCode::CREATED
}

async fn sell_item(State(state): State<AppState>, Path(id): Path<u32>) -> impl IntoResponse {
    let mut data = state.data.lock().await;
    if let Some(item) = data.items.iter_mut().find(|i| i.id == id) {
        if item.quantity > 0 {
            item.quantity -= 1;
            item.sold += 1;
            save_data(&state.file_path, &data);
            return (StatusCode::OK, "Sold".to_string());
        } else {
            return (StatusCode::BAD_REQUEST, "Out of stock".to_string());
        }
    }
    (StatusCode::NOT_FOUND, "Item not found".to_string())
}

async fn upload_image(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name != "image" {
            continue;
        }

        // Get original filename and sanitize extension
        let original_filename = field.file_name().unwrap_or("upload").to_string();
        let ext = std::path::Path::new(&original_filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
            .to_lowercase();

        // Only allow image extensions
        let allowed = ["jpg", "jpeg", "png", "gif", "webp", "avif"];
        if !allowed.contains(&ext.as_str()) {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid file type" })),
            );
        }

        // Generate a unique filename so users can't overwrite files
        let unique_name = format!("{}.{}", Uuid::new_v4(), ext);
        let save_path = format!("static/uploads/{}", unique_name);

        match field.bytes().await {
            Ok(data) => {
                if let Ok(mut file) = File::create(&save_path) {
                    let _ = file.write_all(&data);
                    // Return the public URL path (not the filesystem path)
                    let url = format!("/static/uploads/{}", unique_name);
                    return (
                        StatusCode::OK,
                        axum::Json(serde_json::json!({ "url": url })),
                    );
                }
            }
            Err(_) => {}
        }
    }

    (
        StatusCode::BAD_REQUEST,
        axum::Json(serde_json::json!({ "error": "Upload failed" })),
    )
}
