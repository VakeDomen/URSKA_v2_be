use actix_cors::Cors;
use actix_web::{
    get, web::{self, Bytes, Query}, App, HttpResponse, HttpServer, Responder
};
use serde::Deserialize;
use std::{convert::Infallible, env, fs::{self, create_dir_all, File}, io::Read, sync::Mutex, time::Instant};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::rag::{Rag, RagProcessableFile, RagProcessableFileType};

#[derive(Debug, Deserialize)]
struct SearchQuery {
    query: String,
}

#[get("/search")]
async fn search(search_query: Query<SearchQuery>) -> impl Responder {
    let rag = Rag::default();
    let mut result = match rag.search(search_query.query.clone()).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("{:#?}", e)),
    };

    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(10000);
    let stream = ReceiverStream::new(rx);

    let Ok(chunks_json) = serde_json::to_string(&result.chunks) else {
        return HttpResponse::InternalServerError().finish();
    };
    let _ = tx.send(Bytes::try_from(chunks_json)).await;
    let _ = tx.send(Bytes::try_from("\n")).await;

    actix_web::rt::spawn(async move {
        while let Some(res) = result.stream.next().await {
            if let Ok(responses) = res {
                for resp in responses {
                    let data = Bytes::copy_from_slice(resp.response.as_bytes());
                    let _ = tx.send(Ok(data)).await;
                }
            }
        }
    });

    HttpResponse::Ok().content_type("text/plain").streaming(stream)
}



#[derive(Debug, Deserialize)]
struct BuildQuery {
    query: String,
}

#[get("/build")]
async fn build(search_query: Query<BuildQuery>) -> impl Responder {
    let rag = Rag::default();

    let input_dir = &search_query.query;
    let done_dir = "./resources/done";
    let failed_dir = "./resources/failed";

    let Ok(_) = fs::create_dir_all(done_dir) else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(_) = fs::create_dir_all(failed_dir) else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(files) = fs::read_dir(input_dir) else {
        return HttpResponse::InternalServerError().finish();
    };

    for (id, entry) in files.enumerate() {
        let Ok(entry) = entry else {
            eprintln!("Skipping file, can't find? {:?}", entry);
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            eprintln!("Skipping folder: {:?}", path);
            continue;
        }
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                eprintln!("Skipping file with no valid name: {:?}", path);
                continue;
            }
        };

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_type = match extension.as_str() {
            "pdf" => RagProcessableFileType::Pdf,
            "md" => RagProcessableFileType::Markdown,
            "txt" => RagProcessableFileType::Text,
            _ => RagProcessableFileType::Text,
            
        };

        let woodstock_data = RagProcessableFile {
            path: path.clone(),
            file_type,
            internal_id: format!("{}_{}", id, file_name),
            original_name: file_name.clone(),
            tags: Some(vec![to_link(file_name.clone())]),
            file_description: None,
        };

        let start_time = Instant::now();

        match rag.insert(woodstock_data).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                println!("Successfully inserted file '{}' in {:?}", file_name, duration);
                let done_path = format!("{}/{}", done_dir, file_name);
                if let Err(e) = fs::rename(&path, &done_path) {
                    eprintln!("Failed to move '{}' to done: {}", file_name, e);
                }
            }
            Err(e) => {
                let duration = start_time.elapsed();
                eprintln!("Failed to insert file '{}' in {:?}: {:?}", file_name, duration, e);
            }
        }
    }

    HttpResponse::Ok().into()
}

fn to_link(name: String) -> String {
    if !name.starts_with("https:") {
        return "None".into();
    }

    name
        .replace(":_", "://")
        .replace("_", "/")
        .replace(".md_translated", "")
        .replace(".md", "")
        
}
pub async fn start_server() {
    let server_port = env::var("SERVER_PORT")
        .ok()
        .and_then(|x| x.parse::<u16>().ok())
        .unwrap_or(6969);
    
    create_dir_all(env::var("FILES_FOLDER")
        .unwrap_or("/var/woodstock/files".to_string()))
        .expect("Unable to create the files folder.");

    println!("Server is running on localhost:{}", server_port);
    let _ = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(web::scope("/api")
                .service(search)
                .service(build)
            )
    })
    .bind(("localhost", server_port))
    .expect("Unable to start the server")
    .run()
    .await;
}