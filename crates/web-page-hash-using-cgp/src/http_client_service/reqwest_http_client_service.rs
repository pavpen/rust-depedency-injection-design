use super::interface::{GetUrlService, GetUrlServiceComponent, HasUrlType, IntoChunkStream};
use bytes::Bytes;
use cgp::prelude::*;
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

impl HasUrlType for ReqwestHttpClientService {
    type Url = reqwest::Url;
}

#[cgp_impl(ReqwestHttpClientService)]
impl<Context> GetUrlService for Context
where
    Context: HasUrlType,
    Context::Url: reqwest::IntoUrl + Clone,
{
    type HttpResponse = reqwest::Response;
    type Error = reqwest::Error;

    async fn get_url(
        _context: &Context,
        url: &Context::Url,
    ) -> Result<Self::HttpResponse, Self::Error> {
        let url: Context::Url = url.clone();
        reqwest::get(url).await
    }
}

impl IntoChunkStream for reqwest::Response {
    type Error = reqwest::Error;

    fn into_chunk_stream(self) -> impl Stream<Item = Result<Bytes, Self::Error>> {
        reqwest::Response::bytes_stream(self)
    }
}
