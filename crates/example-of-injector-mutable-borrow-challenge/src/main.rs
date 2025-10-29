use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Allows injecting a mutable reference to a service
///
/// This trait is usually implemented by an injector type.
pub trait InjectMut<'self_lifetime, T> {
    fn inject_mut(&'self_lifetime mut self) -> &'self_lifetime mut T;
}

pub trait CalculateWebPageMessageDigest {
    type Url;
    type Digest;
    type Error;
    type HttpClientService;
    type MessageDigestService;

    fn calculate_web_page_message_digest<
        'fn_call,
        Injector: InjectMut<'fn_call, Self::HttpClientService>
            + InjectMut<'fn_call, Self::MessageDigestService>,
    >(
        &self,
        injector: &'fn_call mut Injector,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::Digest, Self::Error>>
    where
        Self::HttpClientService: 'fn_call,
        Self::MessageDigestService: 'fn_call,
        &'fn_call Injector: Send;
}

struct MockUrl;
struct MockDigest;
struct MockError;
struct MockChunkStream;

impl futures_core::Stream for MockChunkStream {
    type Item = Result<Vec<u8>, MockError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}

struct MockHttpResponse;

impl MockHttpResponse {
    fn into_chunk_stream(self) -> MockChunkStream {
        MockChunkStream
    }
}

struct MockHttpClientService;

impl MockHttpClientService {
    async fn get_url(&self, _url: &MockUrl) -> Result<MockHttpResponse, MockError> {
        Ok(MockHttpResponse)
    }
}

struct MockMessageDigestCalculator;

impl MockMessageDigestCalculator {
    fn write_all(&mut self, _chunk: &Vec<u8>) -> Result<(), MockError> {
        Ok(())
    }

    fn into_digest_octets(self) -> Result<MockDigest, MockError> {
        Ok(MockDigest {})
    }
}

struct MockMessageDigestService;

impl MockMessageDigestService {
    fn new_digest_calculator(&self) -> Result<MockMessageDigestCalculator, MockError> {
        Ok(MockMessageDigestCalculator {})
    }
}

struct MockCalculateWebPageMessageDigestService;

impl CalculateWebPageMessageDigest for MockCalculateWebPageMessageDigestService {
    type Url = MockUrl;
    type Digest = MockDigest;
    type Error = MockError;
    type HttpClientService = MockHttpClientService;
    type MessageDigestService = MockMessageDigestService;

    async fn calculate_web_page_message_digest<
        'fn_call,
        Injector: InjectMut<'fn_call, Self::HttpClientService>
            + InjectMut<'fn_call, Self::MessageDigestService>,
    >(
        &self,
        injector: &'fn_call mut Injector,
        url: &Self::Url,
    ) -> Result<Self::Digest, Self::Error>
    where
        Self::HttpClientService: 'fn_call,
        Self::MessageDigestService: 'fn_call,
        &'fn_call Injector: Send,
    {
        let message_digest_service: &mut Self::MessageDigestService =
            InjectMut::inject_mut(injector);
        let http_client_service: &mut Self::HttpClientService = InjectMut::inject_mut(injector);

        // . . .
        let mut digest_calculator = message_digest_service.new_digest_calculator()?;
        let mut chunk_stream = http_client_service.get_url(url).await?.into_chunk_stream();

        while let Some(chunk_result) = chunk_stream.next().await {
            let chunk = chunk_result?;
            digest_calculator.write_all(&chunk)?;
        }

        Ok(digest_calculator.into_digest_octets()?)
    }
}

fn main() {
    println!("Hello, world!");
}
