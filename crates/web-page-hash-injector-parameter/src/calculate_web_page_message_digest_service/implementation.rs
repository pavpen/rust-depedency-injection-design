use super::interface::CalculateWebPageMessageDigest;
use crate::http_client_service::{GetUrl, IntoChunkStream};
use crate::injector::InjectRef;
use crate::message_digest_service::{IntoDigestOctets, NewDigestCalculator};
use futures_util::StreamExt;
use std::io::Write;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct CalculateWebPageMessageDigestService<
    Url,
    Digest,
    Error,
    HttpClientService: GetUrl<Url = Url>,
    MessageDigestService: NewDigestCalculator + Send,
> {
    _url_type_marker: PhantomData<Url>,
    _digest_type_marker: PhantomData<Digest>,
    _error_type_marker: PhantomData<Error>,
    _http_client_service_type_marker: PhantomData<HttpClientService>,
    _message_digest_service: PhantomData<MessageDigestService>,
}

impl<
    Url,
    Digest,
    Error,
    HttpClientService: GetUrl<Url = Url>,
    MessageDigestService: NewDigestCalculator + Send,
>
    CalculateWebPageMessageDigestService<
        Url,
        Digest,
        Error,
        HttpClientService,
        MessageDigestService,
    >
{
    pub fn new() -> Self {
        CalculateWebPageMessageDigestService {
            _url_type_marker: PhantomData::<Url>,
            _digest_type_marker: PhantomData::<Digest>,
            _error_type_marker: PhantomData::<Error>,
            _http_client_service_type_marker: PhantomData::<HttpClientService>,
            _message_digest_service: PhantomData::<MessageDigestService>,
        }
    }
}

impl<
    Url:Send+Sync,
    Digest: Send+Sync,
    Error: Sync,
    HttpClientService: GetUrl<Url = Url>+Sync,
    MessageDigestService: NewDigestCalculator + Send+Sync,
> CalculateWebPageMessageDigest
    for CalculateWebPageMessageDigestService<
        Url,
        Digest,
        Error,
        HttpClientService,
        MessageDigestService,
    >
where
    Error: From<<HttpClientService as GetUrl>::Error>
        + From<<<HttpClientService as GetUrl>::HttpResponse as IntoChunkStream>::Error>
        + From<<MessageDigestService as NewDigestCalculator>::Error>
        + From<<<MessageDigestService as NewDigestCalculator>::DigestCalculator as IntoDigestOctets>::Error>
        + From<std::io::Error>
        + Send,
    HttpClientService::HttpResponse: IntoChunkStream,
    <HttpClientService as GetUrl>::HttpResponse: Sync,
    <HttpClientService as GetUrl>::Error: Sync,
    MessageDigestService::DigestCalculator: IntoDigestOctets<DigestOctets = Digest> + Write + Send + Sync,
{
    type Url = Url;
    type Digest = Digest;
    type Error = Error;
    type HttpClientService = HttpClientService;
    type MessageDigestService = MessageDigestService;

    async fn calculate_web_page_message_digest<
        'fn_call,
        Injector: InjectRef<'fn_call, HttpClientService> + InjectRef<'fn_call, MessageDigestService>,
    >(
        &self,
        injector: &'fn_call Injector,
        url: &Self::Url,
    ) -> Result<Self::Digest, Self::Error>
    where
        HttpClientService: 'fn_call,
        MessageDigestService: 'fn_call,
        &'fn_call Injector: Send
    {
        let message_digest_service: &MessageDigestService = InjectRef::inject_ref(injector);
        let mut digest_calculator = message_digest_service.new_digest_calculator()?;
        let http_client_service: &HttpClientService = InjectRef::inject_ref(injector);
        let mut chunk_stream = http_client_service
            .get_url(url)
            .await?
            .into_chunk_stream();

        while let Some(chunk_result) = chunk_stream.next().await {
            let chunk = chunk_result?;
            digest_calculator.write_all(&chunk)?;
        }

        Ok(digest_calculator.into_digest_octets()?)
    }
}
