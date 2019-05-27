/*!
# File Center Download Response on MongoDB for Rocket Framework

This crate provides a response struct used for client downloading from the File Center on MongoDB.

See `examples`.
*/

pub extern crate mongo_file_center;
extern crate percent_encoding;
extern crate rocket;

use std::io::Cursor;

use mongo_file_center::{FileCenter, FileItem, FileData, FileCenterError, bson::oid::ObjectId};

use rocket::response::{self, Response, Responder};
use rocket::request::Request;

/// The response struct used for responding raw data from the File Center on MongoDB with **Etag** cache.
#[derive(Debug)]
pub struct FileCenterDownloadResponse {
    file_name: Option<String>,
    file_item: FileItem,
}

impl FileCenterDownloadResponse {
    /// Create a `FileCenterDownloadResponse` instance from the object ID.
    pub fn from_object_id<S: Into<String>>(file_center: &FileCenter, id: &ObjectId, file_name: Option<S>) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
        let file_item = file_center.get_file_item_by_id(id)?;

        match file_item {
            Some(file_item) => {
                let file_name = file_name.map(|file_name| file_name.into());

                Ok(Some(FileCenterDownloadResponse {
                    file_name,
                    file_item,
                }))
            }
            None => Ok(None)
        }
    }

    /// Create a `FileCenterDownloadResponse` instance from an ID token.
    pub fn from_id_token<T: AsRef<str> + Into<String>, S: Into<String>>(file_center: &FileCenter, id_token: T, file_name: Option<S>) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
        let id = file_center.decrypt_id_token(id_token.as_ref())?;

        Self::from_object_id(file_center, &id, file_name)
    }
}

impl Responder<'static> for FileCenterDownloadResponse {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        let mut response = Response::build();

        let file_name = self.file_name.as_ref().map(|file_name| file_name.as_str()).unwrap_or(self.file_item.get_file_name());

        if file_name.is_empty() {
            response.raw_header("Content-Disposition", "attachment");
        } else {
            response.raw_header("Content-Disposition", format!("attachment; filename*=UTF-8''{}", percent_encoding::percent_encode(file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
        }

        response.raw_header("Content-Type", self.file_item.get_mime_type().to_string());

        let file_size = self.file_item.get_file_size();

        match self.file_item.into_file_data() {
            FileData::Collection(v) => {
                response.sized_body(Cursor::new(v));
            }
            FileData::GridFS(g) => {
                response.raw_header("Content-Length", file_size.to_string());

                response.streamed_body(g);
            }
        }

        response.ok()
    }
}