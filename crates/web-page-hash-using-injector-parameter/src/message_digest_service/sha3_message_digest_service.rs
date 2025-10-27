use super::interface::{IntoDigestOctets, MessageDigestService, NewDigestCalculator};
use sha3;
use sha3::Digest;
use std::convert::Infallible;

impl IntoDigestOctets for sha3::Sha3_256 {
    type DigestOctets = [u8; 32];
    type Error = Infallible;

    fn into_digest_octets(self) -> Result<Self::DigestOctets, Self::Error> {
        Ok(sha3::Sha3_256::finalize(self).into())
    }
}

#[derive(Debug, Default)]
pub struct Sha3_256BitMessageDigestService {}

impl Sha3_256BitMessageDigestService {
    pub fn new() -> Self {
        Sha3_256BitMessageDigestService {}
    }
}

impl NewDigestCalculator for Sha3_256BitMessageDigestService {
    type DigestCalculator = sha3::Sha3_256;
    type Error = Infallible;

    fn new_digest_calculator(&self) -> Result<Self::DigestCalculator, Self::Error> {
        Ok(sha3::Sha3_256::new())
    }
}

impl MessageDigestService for Sha3_256BitMessageDigestService {}
