#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

extern crate rocket_mongo_file_center_download_response;

extern crate validators;

use std::path::Path;

use rocket_mongo_file_center_download_response::mongo_file_center::{
    mime, FileCenter, FileCenterError,
};
use rocket_mongo_file_center_download_response::FileCenterDownloadResponse;

use rocket::request::State;

use validators::short_crypt_url_component::ShortCryptUrlComponent;

const URI: &str = "mongodb://localhost:27017/test_rocket_mongo_file_center_download_response";

#[get("/<id_token>")]
fn download(
    file_center: State<FileCenter>,
    id_token: ShortCryptUrlComponent,
) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
    FileCenterDownloadResponse::from_id_token(
        file_center.inner(),
        id_token.into_string(),
        None::<String>,
    )
}

fn main() {
    let file_center = FileCenter::new(URI).unwrap();

    let path = Path::join(Path::new("examples"), Path::join(Path::new("images"), "image(貓).jpg"));

    let file = file_center
        .put_file_by_path_temporarily(path, None::<String>, Some(mime::IMAGE_JPEG))
        .unwrap();

    let id_token = file_center.encrypt_id(file.get_object_id());

    println!("The ID token is: {}", id_token);

    println!();

    rocket::ignite().manage(file_center).mount("/", routes![download]).launch();
}
