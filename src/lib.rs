/*!
# File Center Download Response on MongoDB for Rocket Framework

This crate provides a response struct used for client downloading from the File Center on MongoDB.

See `examples`.
*/

pub extern crate mongo_file_center;
extern crate percent_encoding;
extern crate rocket;

use std::io::Cursor;
use std::fmt::{self, Debug, Formatter};

use mongo_file_center::{FileCenter, FileItem, FileData, FileCenterError, bson::oid::ObjectId};

use rocket::response::{self, Response, Responder};
use rocket::request::Request;

/// The response struct used for client downloading from the File Center on MongoDB.
pub struct FileCenterDownloadResponse {
    pub file_item: FileItem,
    pub file_name: Option<String>,
}

impl Debug for FileCenterDownloadResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("FileCenterDownloadResponse {{file_name: {:?}, file_item: {:?}}}", self.file_name, self.file_item))
    }
}

impl Responder<'static> for FileCenterDownloadResponse {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        let mut response = Response::build();

        response.raw_header("Content-Transfer-Encoding", "binary");

        let file_item = self.file_item;

        if let Some(file_name) = self.file_name {
            if file_name.is_empty() {
                response.raw_header("Content-Disposition", "attachment");
            } else {
                response.raw_header("Content-Disposition", format!("attachment; filename*=UTF-8''{}", percent_encoding::percent_encode(file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
            }
        } else {
            let file_name = file_item.get_file_name();
            if file_name.is_empty() {
                response.raw_header("Content-Disposition", "attachment");
            } else {
                response.raw_header("Content-Disposition", format!("attachment; filename*=UTF-8''{}", percent_encoding::percent_encode(file_name.as_bytes(), percent_encoding::QUERY_ENCODE_SET)));
            }
        }

        response.raw_header("Content-Type", file_item.get_mime_type().to_string())
            .raw_header("Content-Length", file_item.get_file_size().to_string());

        match file_item.into_file_data() {
            FileData::Collection(v) => {
                response.sized_body(Cursor::new(v));
            }
            FileData::GridFS(g) => {
                response.streamed_body(g);
            }
        }

        response.ok()
    }
}

impl FileCenterDownloadResponse {
    /// Create a `FileCenterDownloadResponse` instance from a file item.
    pub fn from_object_id<S: Into<String>>(file_center: &FileCenter, id: &ObjectId, file_name: Option<S>) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
        let file_item = file_center.get_file_item_by_id(id)?;

        match file_item {
            Some(file_item) => {
                let file_name = file_name.map(|file_name| file_name.into());
                Ok(Some(FileCenterDownloadResponse {
                    file_item,
                    file_name,
                }))
            }
            None => Ok(None)
        }
    }
}