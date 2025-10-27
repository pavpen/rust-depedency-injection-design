/// See <https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/>, and
/// <https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/>
/// for more backgroun on traits containing `async` functions.
#[trait_variant::make(CalculateWebPageMessageDigest: Send)]
pub trait ThreadLocalCalculateWebPageMessageDigest {
    type Url;
    type Digest;
    type Error;

    fn calculate_web_page_message_digest(
        &self,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::Digest, Self::Error>>;
}
