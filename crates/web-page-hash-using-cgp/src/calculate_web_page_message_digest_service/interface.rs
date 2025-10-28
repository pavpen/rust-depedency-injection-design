use crate::http_client_service::HasUrlType;
use cgp::prelude::*;

#[cgp_type]
pub trait HasDigestType {
    type Digest;
}

#[cgp_component(CalculateWebPageMessageDigestService)]
pub trait CalculateWebPageMessageDigest: HasUrlType + HasDigestType + HasErrorType {
    fn calculate_web_page_message_digest(
        &self,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::Digest, Self::Error>>;
}
