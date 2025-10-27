use clap::Parser;
use web_page_hash_explicit_arguments::{
    calculate_web_page_message_digest_service::{
        CalculateWebPageMessageDigest, CalculateWebPageMessageDigestService,
    },
    http_client_service::ReqwestHttpClientService,
    message_digest_service::Sha3_256BitMessageDigestService,
};

/// Prints the 256-bit SHA-3 message digest of a Web page
///
/// This is a tiny demo app using a design option for Depedency Injection in
/// Rust.
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(long)]
    url: String,
}

struct HexFormatted<'a>(&'a [u8]);

impl std::fmt::LowerHex for HexFormatted<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for octet in self.0 {
            write!(f, "{:x}", octet)?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let url = reqwest::Url::parse(&args.url)?;
    let http_client_service = ReqwestHttpClientService::new();
    let message_digest_service = Sha3_256BitMessageDigestService::new();
    let calculate_web_page_message_digest_service =
        CalculateWebPageMessageDigestService::<
            reqwest::Url,
            [u8; 32],
            anyhow::Error,
            ReqwestHttpClientService,
            Sha3_256BitMessageDigestService,
        >::new(http_client_service, message_digest_service);

    println!("Hashing content: {}", url);
    let digest = calculate_web_page_message_digest_service
        .calculate_web_page_message_digest(&url)
        .await?;
    println!("256-bit SHA-3: 0x{:x}", HexFormatted(&digest));

    Ok(())
}
