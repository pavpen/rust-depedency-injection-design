/// An object that can be converted into an array of message digest octets
///
/// The object can be, e.g., a 256-bit SHA-3 calculator to which input message
/// data can be written.  When the full message has been written the
/// calculator object can be converted into a 256-bit array of message digest
/// octets.
pub trait IntoDigestOctets {
    type DigestOctets;
    type Error;

    fn into_digest_octets(self) -> Result<Self::DigestOctets, Self::Error>;
}

pub trait NewDigestCalculator {
    type DigestCalculator;
    type Error;

    fn new_digest_calculator(&self) -> Result<Self::DigestCalculator, Self::Error>;
}

// The full service is composed of its method, and function traits:
pub trait MessageDigestService: NewDigestCalculator
where
    <Self as NewDigestCalculator>::DigestCalculator: IntoDigestOctets + std::io::Write,
{
}
