use crate::injector::InjectRef;

/// See <https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/>, and
/// <https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/>
/// for more backgroun on traits containing `async` functions.
#[trait_variant::make(CalculateWebPageMessageDigest: Send)]
pub trait ThreadLocalCalculateWebPageMessageDigest {
    type Url;
    type Digest;
    type Error;
    type HttpClientService;
    type MessageDigestService;

    fn calculate_web_page_message_digest<
        'fn_call,
        Injector: InjectRef<'fn_call, Self::HttpClientService>
            + InjectRef<'fn_call, Self::MessageDigestService>,
    >(
        &self,
        injector: &'fn_call Injector,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::Digest, Self::Error>>
    where
        Self::HttpClientService: 'fn_call,
        Self::MessageDigestService: 'fn_call,
        &'fn_call Injector: Send;
}
