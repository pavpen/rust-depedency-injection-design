use super::interface::{GetUrl, IntoChunkStream};
use bytes::Bytes;
use futures_core::stream::Stream;
use reqwest;

/// An injectable HTTP client service implemented using the [reqwest] crate
#[derive(Debug, Default)]
pub struct ReqwestHttpClientService {}

impl ReqwestHttpClientService {
    pub fn new() -> Self {
        ReqwestHttpClientService {}
    }
}

impl GetUrl for ReqwestHttpClientService {
    type Url = reqwest::Url;
    type HttpResponse = reqwest::Response;
    type Error = reqwest::Error;

    async fn get_url(&self, url: &Self::Url) -> Result<Self::HttpResponse, Self::Error> {
        reqwest::get(url.clone()).await
    }
}

impl IntoChunkStream for reqwest::Response {
    type Error = reqwest::Error;

    fn into_chunk_stream(self) -> impl Stream<Item = Result<Bytes, Self::Error>> {
        reqwest::Response::bytes_stream(self)
    }
}
