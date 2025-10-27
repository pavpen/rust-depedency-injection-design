use super::interface::CalculateWebPageMessageDigest;
use crate::http_client_service::{GetUrl, IntoChunkStream};
use crate::message_digest_service::{IntoDigestOctets, NewDigestCalculator};
use futures_util::StreamExt;
use std::io::Write;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct CalculateWebPageMessageDigestService<
    Url,
    Digest,
    Error: Sync,
    HttpClientService: GetUrl<Url = Url>,
    MessageDigestService: NewDigestCalculator + Send,
> {
    http_client_service: HttpClientService,
    message_digest_service: MessageDigestService,
    _digest_type_marker: PhantomData<Digest>,
    _error_type_marker: PhantomData<Error>,
}

impl<
    Url,
    Digest,
    Error: Sync,
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
    pub fn new(
        http_client_service: HttpClientService,
        message_digest_service: MessageDigestService,
    ) -> Self {
        CalculateWebPageMessageDigestService::<
            Url,
            Digest,
            Error,
            HttpClientService,
            MessageDigestService,
        > {
            http_client_service,
            message_digest_service,
            _digest_type_marker: PhantomData,
            _error_type_marker: PhantomData,
        }
    }
}

impl<
    Url:Sync,
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

    async fn calculate_web_page_message_digest(
        &self,
        url: &Self::Url,
    ) -> Result<Self::Digest, Self::Error> {
        let mut digest_calculator = self.message_digest_service.new_digest_calculator()?;
        let http_client_service = &self.http_client_service;
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
