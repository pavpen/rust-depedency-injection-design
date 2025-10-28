use super::interface::{
    IntoDigestOctets, NewDigestCalculatorService, NewDigestCalculatorServiceComponent,
};
use cgp::prelude::*;
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

#[cgp_provider]
impl<Context> NewDigestCalculatorService<Context> for Sha3_256BitMessageDigestService {
    type DigestCalculator = sha3::Sha3_256;
    type Error = Infallible;

    fn new_digest_calculator(_context: &Context) -> Result<Self::DigestCalculator, Self::Error> {
        Ok(sha3::Sha3_256::new())
    }
}
