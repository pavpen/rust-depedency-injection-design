use clap::Parser;
use web_page_hash_injector_parameter::{
    calculate_web_page_message_digest_service::{self, CalculateWebPageMessageDigest},
    http_client_service::{GetUrl, ReqwestHttpClientService},
    injector::InjectRef,
    message_digest_service::{
        IntoDigestOctets, NewDigestCalculator, Sha3_256BitMessageDigestService,
    },
};

type HttpClientService = ReqwestHttpClientService;
type MessageDigestService = Sha3_256BitMessageDigestService;
type Url = <HttpClientService as GetUrl>::Url;
type Digest = <<MessageDigestService as NewDigestCalculator>::DigestCalculator as IntoDigestOctets>::DigestOctets;
type CalculateWebPageMessageDigestService =
    calculate_web_page_message_digest_service::CalculateWebPageMessageDigestService<
        Url,
        Digest,
        anyhow::Error,
        HttpClientService,
        MessageDigestService,
    >;

/// Contains services that don't depend on other services
struct Stage1Injector {
    http_client_service: HttpClientService,
    message_digest_service: MessageDigestService,
}

impl Stage1Injector {
    fn from_services(
        http_client_service: HttpClientService,
        message_digest_service: MessageDigestService,
    ) -> Self {
        Stage1Injector {
            http_client_service,
            message_digest_service,
        }
    }
}

impl<'self_lifetime> InjectRef<'self_lifetime, HttpClientService> for Stage1Injector {
    fn inject_ref(&'self_lifetime self) -> &'self_lifetime HttpClientService {
        &self.http_client_service
    }
}

impl<'self_lifetime> InjectRef<'self_lifetime, MessageDigestService> for Stage1Injector {
    fn inject_ref(&'self_lifetime self) -> &'self_lifetime MessageDigestService {
        &self.message_digest_service
    }
}

/// Contains a [Stage1Injector], and services that depend on [Stage1Injector]
struct Stage2Injector {
    dependency_injector: Stage1Injector,
    calculate_web_page_message_digest_service: CalculateWebPageMessageDigestService,
}

impl Stage2Injector {
    fn from_dependency_injector_and_services(
        dependency_injector: Stage1Injector,
        calculate_web_page_message_digest_service: CalculateWebPageMessageDigestService,
    ) -> Self {
        Stage2Injector {
            dependency_injector,
            calculate_web_page_message_digest_service,
        }
    }
}

impl<'self_lifetime> InjectRef<'self_lifetime, HttpClientService> for Stage2Injector {
    fn inject_ref(&'self_lifetime self) -> &'self_lifetime HttpClientService {
        InjectRef::inject_ref(&self.dependency_injector)
    }
}

impl<'self_lifetime> InjectRef<'self_lifetime, MessageDigestService> for Stage2Injector {
    fn inject_ref(&'self_lifetime self) -> &'self_lifetime MessageDigestService {
        InjectRef::inject_ref(&self.dependency_injector)
    }
}

impl<'self_lifetime> InjectRef<'self_lifetime, CalculateWebPageMessageDigestService>
    for Stage2Injector
{
    fn inject_ref(&'self_lifetime self) -> &'self_lifetime CalculateWebPageMessageDigestService {
        &self.calculate_web_page_message_digest_service
    }
}

/// Prints the 256-bit SHA-3 message digest of a Web page
///
/// This is a tiny demo app using the injector parameter design option for
/// Depedency Injection in Rust.
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
    let stage1_injector = Stage1Injector::from_services(
        ReqwestHttpClientService::new(),
        Sha3_256BitMessageDigestService::new(),
    );
    let injector = Stage2Injector::from_dependency_injector_and_services(
        stage1_injector,
        CalculateWebPageMessageDigestService::new(),
    );
    let calculate_web_page_message_digest_service: &CalculateWebPageMessageDigestService =
        InjectRef::inject_ref(&injector);

    println!("Fetching: {}", url);
    let digest = calculate_web_page_message_digest_service
        .calculate_web_page_message_digest(&injector, &url)
        .await?;
    println!("256-bit SHA-3: 0x{:x}", HexFormatted(&digest));

    Ok(())
}
