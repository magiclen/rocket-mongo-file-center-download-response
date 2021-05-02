#[macro_use]
extern crate rocket;

extern crate rocket_mongo_file_center_download_response;

#[macro_use]
extern crate validators_derive;

use std::error::Error;
use std::path::Path;

use rocket_mongo_file_center_download_response::mongo_file_center::{mime, FileCenter};
use rocket_mongo_file_center_download_response::FileCenterDownloadResponse;

use rocket::http::Status;
use rocket::State;

use validators::prelude::*;

const URI: &str = "mongodb://localhost:27017/test_rocket_mongo_file_center_download_response";

#[derive(Debug, Clone, Validator)]
#[validator(base64_url(padding(NotAllow)))]
struct ShortCryptUrlComponent(pub(crate) String);

#[get("/<id_token>")]
fn download(
    file_center: State<FileCenter>,
    id_token: ShortCryptUrlComponent,
) -> Result<Option<FileCenterDownloadResponse>, Status> {
    FileCenterDownloadResponse::from_id_token(file_center.inner(), id_token.0, None::<String>)
        .map_err(|_| Status::InternalServerError)
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_center = FileCenter::new(URI)?;

    let path = Path::join(Path::new("examples"), Path::join(Path::new("images"), "image(貓).jpg"));

    let file =
        file_center.put_file_by_path_temporarily(path, None::<String>, Some(mime::IMAGE_JPEG))?;

    let id_token = file_center.encrypt_id(file.get_object_id());

    println!("The ID token is: {}", id_token);

    println!();

    rocket::build().manage(file_center).mount("/", routes![download]).launch().await?;

    Ok(())
}
