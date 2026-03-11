use Shop::{load_data, save_data, Item, Sale, StoredData};
use axum::{
    Router,
    extract::{Multipart, Path, State, Json},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json as AxumJson,
};
use chrono::Utc;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Deserialize)]
struct NewItem {
    image: String,
    name: String,
    price: f64,
    quantity: u32,
}

#[derive(Deserialize)]
struct SellRequest {
    quantity: u32,
}

#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<StoredData>>,
}

#[tokio::main]
async fn main() {
    // Ensure uploads directory exists
    fs::create_dir_all("static/uploads").unwrap();

    let stored_data = load_data();

    let state = AppState {
        data: Arc::new(Mutex::new(stored_data)),
    };

    let app = Router::new()
        .route("/", get(serve_page))
        .route("/items", get(get_items).post(add_item))
        .route("/items/:id/sell", post(sell_item))
        .route("/sales", get(get_sales))
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
    AxumJson(data.items.clone())
}

async fn get_sales(State(state): State<AppState>) -> impl IntoResponse {
    let data = state.data.lock().await;
    AxumJson(data.sales.clone())
}

async fn add_item(
    State(state): State<AppState>,
    Json(new_item): Json<NewItem>,
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
    save_data(&data);
    StatusCode::CREATED
}

async fn sell_item(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<SellRequest>,
) -> impl IntoResponse {
    let mut data = state.data.lock().await;

    let next_sale_id = data.next_sale_id;
    if let Some(item) = data.items.iter_mut().find(|i| i.id == id) {
        if item.quantity >= req.quantity && req.quantity > 0 {
            item.quantity -= req.quantity;
            item.sold += req.quantity;

            let sale = Sale {
                id: next_sale_id,
                item_id: item.id,
                item_name: item.name.clone(),
                quantity: req.quantity,
                price_at_sale: item.price,
                timestamp: Utc::now(),
            };
            data.sales.push(sale);
            data.next_sale_id += 1;

            save_data(&data);
            return (StatusCode::OK, "Sold".to_string()).into_response();
        } else if req.quantity == 0 {
            return (StatusCode::BAD_REQUEST, "Quantity must be > 0".to_string()).into_response();
        } else {
            return (StatusCode::BAD_REQUEST, "Not enough stock".to_string()).into_response();
        }
    }
    (StatusCode::NOT_FOUND, "Item not found".to_string()).into_response()
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
                AxumJson(serde_json::json!({ "error": "Invalid file type" })),
            ).into_response();
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
                        AxumJson(serde_json::json!({ "url": url })),
                    ).into_response();
                }
            }
            Err(_) => {}
        }
    }

    (
        StatusCode::BAD_REQUEST,
        AxumJson(serde_json::json!({ "error": "Upload failed" })),
    ).into_response()
}
