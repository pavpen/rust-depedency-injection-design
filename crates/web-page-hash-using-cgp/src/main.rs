use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;
use cgp_error_anyhow::UseAnyhowError;
use clap::Parser;
use web_page_hash_using_cgp::{
    calculate_web_page_message_digest_service::{
        CalculateWebPageMessageDigest, CalculateWebPageMessageDigestServiceComponent,
        CalculateWebPageMessageDigestServiceObject, DigestTypeProviderComponent,
    },
    http_client_service::{
        GetUrlServiceComponent, ReqwestHttpClientService, UrlTypeProviderComponent,
    },
    message_digest_service::{
        NewDigestCalculatorServiceComponent, Sha3_256BitMessageDigestService,
    },
};

/// Prints the 256-bit SHA-3 message digest of a Web page
///
/// This is a tiny demo app using the Context-Generic Programming design
/// option for Depedency Injection in Rust.
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(long)]
    url: String,
}

type Digest = [u8; 32];

#[cgp_context]
#[derive(Debug, Default)]
struct Services;

impl Services {
    fn new() -> Self {
        Services {}
    }
}

delegate_and_check_components! {
    CanUseServices for Services;
    ServicesComponents {
        ErrorTypeProviderComponent: UseAnyhowError,
        NewDigestCalculatorServiceComponent: Sha3_256BitMessageDigestService,
        UrlTypeProviderComponent: UseType<reqwest::Url>,
        DigestTypeProviderComponent: UseType<Digest>,
        GetUrlServiceComponent: ReqwestHttpClientService,
        CalculateWebPageMessageDigestServiceComponent:
            CalculateWebPageMessageDigestServiceObject,
    }
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
    let services = Services::new();

    println!("Fetching: {}", url);
    let digest = services.calculate_web_page_message_digest(&url).await?;
    println!("256-bit SHA-3: 0x{:x}", HexFormatted(&digest));

    Ok(())
}
