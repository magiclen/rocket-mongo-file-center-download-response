#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

extern crate rocket_mongo_file_center_download_response;

extern crate validators;

use std::path::Path;

use rocket_mongo_file_center_download_response::FileCenterDownloadResponse;
use rocket_mongo_file_center_download_response::mongo_file_center::{FileCenter, FileCenterError, mime};

use rocket::request::State;

use validators::short_crypt_url_component::ShortCryptUrlComponent;

const HOST: &str = "localhost";
const PORT: u16 = 27017;

#[get("/<id_token>")]
fn download(file_center: State<FileCenter>, id_token: ShortCryptUrlComponent) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
    let id = file_center.decrypt_id_token(id_token.get_short_crypt_url_component())?;

    FileCenterDownloadResponse::from_object_id(file_center.inner(), &id, None::<String>)
}

fn main() {
    let database = "test_rocket_mongo_file_center_download_response";

    let file_center = FileCenter::new(HOST, PORT, database).unwrap();

    let path = Path::join(Path::new("examples"), Path::join(Path::new("images"), "image(貓).jpg"));

    let file = file_center.put_file_by_path_temporarily(path, None::<String>, Some(mime::IMAGE_JPEG)).unwrap();

    let id_token = file_center.encrypt_id(file.get_object_id());

    println!("The ID token is: {}", id_token);

    println!();

    rocket::ignite().manage(file_center).mount("/", routes![download]).launch();
}