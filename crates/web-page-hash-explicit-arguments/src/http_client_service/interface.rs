use bytes::Bytes;
use futures_core::stream::Stream;

#[trait_variant::make(IntoChunkStream: Send)]
pub trait ThreadLocalIntoChunkStream {
    type Error;

    fn into_chunk_stream(self) -> impl Stream<Item = Result<Bytes, Self::Error>> + Unpin;
}

/// A function trait providing a `get_url` function for [HttpClientService]
///
/// See <https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/>, and
/// <https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/>
/// for more backgroun on traits containing `async` functions.
#[trait_variant::make(GetUrl: Send)]
pub trait ThreadLocalGetUrl {
    type Url;
    type HttpResponse;
    type Error;

    fn get_url(
        &self,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::HttpResponse, Self::Error>>;
}

// The full service is composed of its method, and function traits:
/// An injectable service that can fetch data with HTTP GET requests
pub trait HttpClientService: GetUrl
where
    // We want results returned by `get_url` to be convertible into a [Stream]
    // of [Bytes]:
    <Self as GetUrl>::HttpResponse: IntoChunkStream,
{
}
