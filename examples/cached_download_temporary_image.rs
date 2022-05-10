#[macro_use]
extern crate rocket;

mod common;

use std::error::Error;
use std::path::Path;

use rocket_mongo_file_center_download_response::mongo_file_center::{mime, FileCenter};
use rocket_mongo_file_center_download_response::FileCenterDownloadResponse;

use rocket::http::Status;
use rocket::State;

use common::*;

#[get("/<id_token>")]
async fn download(
    file_center: &State<FileCenter>,
    id_token: ShortCryptUrlComponent,
) -> Result<Option<FileCenterDownloadResponse>, Status> {
    FileCenterDownloadResponse::from_id_token(file_center.inner(), id_token.0, None::<String>)
        .await
        .map_err(|_| Status::InternalServerError)
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_center =
        FileCenter::new(get_mongodb_uri("test_rocket_mongo_file_center_download_response")).await?;

    let path = Path::new("examples").join("images").join("image(貓).jpg");

    let file_id = file_center
        .put_file_by_path_temporarily(path, None::<String>, Some(mime::IMAGE_JPEG))
        .await?;

    let id_token = file_center.encrypt_id(file_id);

    println!("The ID token is: {}", id_token);

    println!();

    let _ = rocket::build().manage(file_center).mount("/", routes![download]).launch().await?;

    Ok(())
}
