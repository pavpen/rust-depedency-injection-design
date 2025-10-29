use super::interface::{
    CalculateWebPageMessageDigestService, CalculateWebPageMessageDigestServiceComponent,
    HasDigestType,
};
use crate::http_client_service::{GetUrl, HasUrlType, IntoChunkStream};
use crate::message_digest_service::{IntoDigestOctets, NewDigestCalculator};
use cgp::prelude::*;
use futures_util::StreamExt;
use std::io::Write;

pub struct CalculateWebPageMessageDigestServiceObject;

#[cgp_impl(CalculateWebPageMessageDigestServiceObject)]
impl<Context> CalculateWebPageMessageDigestService for Context
where
    Context: HasUrlType + HasDigestType + HasErrorType + GetUrl + NewDigestCalculator,
    <Context as HasErrorType>::Error: From<<Context as GetUrl>::Error>
        + From<<<Context as GetUrl>::HttpResponse as IntoChunkStream>::Error>
        + From<<Context as NewDigestCalculator>::Error>
        + From<<<Context as NewDigestCalculator>::DigestCalculator as IntoDigestOctets>::Error>
        + From<std::io::Error>
        + Send,
    <Context as GetUrl>::HttpResponse: IntoChunkStream,
    <Context as GetUrl>::HttpResponse: Sync,
    <Context as GetUrl>::Error: Sync,
    <Context as NewDigestCalculator>::DigestCalculator:
        IntoDigestOctets<DigestOctets = Context::Digest> + Write + Send + Sync,
{
    async fn calculate_web_page_message_digest(
        context: &Context,
        url: &Context::Url,
    ) -> Result<<Context as HasDigestType>::Digest, <Context as HasErrorType>::Error> {
        let mut digest_calculator = context.new_digest_calculator()?;
        let mut chunk_stream = context.get_url(url).await?.into_chunk_stream();

        while let Some(chunk_result) = chunk_stream.next().await {
            let chunk = chunk_result?;
            digest_calculator.write_all(&chunk)?;
        }

        Ok(digest_calculator.into_digest_octets()?)
    }
}
