/*!
# File Center Download Response on MongoDB for Rocket Framework

This crate provides a response struct used for client downloading from the File Center on MongoDB.

See `examples`.
*/

pub extern crate mongo_file_center;

use std::io::Cursor;

use tokio_util::io::StreamReader;

use mongo_file_center::bson::oid::ObjectId;
use mongo_file_center::{FileCenter, FileCenterError, FileData, FileItem};

use rocket::request::Request;
use rocket::response::{self, Responder, Response};

/// The response struct used for client downloading from the File Center on MongoDB.
#[derive(Debug)]
pub struct FileCenterDownloadResponse {
    file_name: Option<String>,
    file_item: FileItem,
}

impl FileCenterDownloadResponse {
    /// Create a `FileCenterDownloadResponse` instance from a file item.
    #[inline]
    pub fn from_file_item<S: Into<String>>(
        file_item: FileItem,
        file_name: Option<S>,
    ) -> FileCenterDownloadResponse {
        let file_name = file_name.map(|file_name| file_name.into());

        FileCenterDownloadResponse {
            file_name,
            file_item,
        }
    }

    /// Create a `FileCenterDownloadResponse` instance from the object ID.
    pub async fn from_object_id<S: Into<String>>(
        file_center: &FileCenter,
        id: ObjectId,
        file_name: Option<S>,
    ) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
        let file_item = file_center.get_file_item_by_id(id).await?;

        match file_item {
            Some(file_item) => Ok(Some(Self::from_file_item(file_item, file_name))),
            None => Ok(None),
        }
    }

    /// Create a `FileCenterDownloadResponse` instance from an ID token.
    #[inline]
    pub async fn from_id_token<I: AsRef<str> + Into<String>, S: Into<String>>(
        file_center: &FileCenter,
        id_token: I,
        file_name: Option<S>,
    ) -> Result<Option<FileCenterDownloadResponse>, FileCenterError> {
        let id = file_center.decrypt_id_token(id_token.as_ref())?;

        Self::from_object_id(file_center, id, file_name).await
    }

    #[inline]
    /// Check if the file item is temporary.
    pub fn is_temporary(&self) -> bool {
        self.file_item.get_expiration_time().is_some()
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for FileCenterDownloadResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let mut response = Response::build();

        let file_name = self.file_name.as_deref().unwrap_or_else(|| self.file_item.get_file_name());

        if file_name.is_empty() {
            response.raw_header("Content-Disposition", "attachment");
        } else {
            let mut v = String::from("attachment; filename*=UTF-8''");

            url_escape::encode_component_to_string(file_name, &mut v);

            response.raw_header("Content-Disposition", v);
        }

        response.raw_header("Content-Type", self.file_item.get_mime_type().to_string());

        let file_size = self.file_item.get_file_size();

        match self.file_item.into_file_data() {
            FileData::Buffer(v) => {
                response.sized_body(v.len(), Cursor::new(v));
            }
            FileData::Stream(g) => {
                response.raw_header("Content-Length", file_size.to_string());

                response.streamed_body(StreamReader::new(g));
            }
        }

        response.ok()
    }
}
