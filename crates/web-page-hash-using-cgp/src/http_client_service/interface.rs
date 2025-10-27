use bytes::Bytes;
use cgp::prelude::*;
use futures_core::stream::Stream;

pub trait IntoChunkStream {
    type Error;

    fn into_chunk_stream(self) -> impl Stream<Item = Result<Bytes, Self::Error>> + Unpin;
}

#[cgp_type]
pub trait HasUrlType {
    type Url;
}

/// A function trait providing a `get_url` function for [HttpClientService]
#[cgp_component(GetUrlService)]
pub trait GetUrl: HasUrlType {
    type HttpResponse;
    type Error;

    fn get_url(
        &self,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::HttpResponse, Self::Error>>;
}
