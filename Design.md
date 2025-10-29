# Dependency Injection in Rust Design

This document considers several designs for implementing
[Dependency Injection](https://martinfowler.com/articles/injection.html) in Rust.

## Goals

Listed in [§Desired Features in Requirements.md](Requirements.md#desired-features)

## Existing Approaches

* [Context-Generic Programming Patterns (DRAFT)](https://patterns.contextgeneric.dev/introduction.html)[^1]
  * A service is defined as a trait.
  * CGP prefers to inject individual functions, rather than services grouping a
    number of them.  This allows partial implementations of 'a traditional'
    service that can be used by a client which requires only functions in the
    partial implementation.
  * Another trait
    (['DelegateComponent'](https://patterns.contextgeneric.dev/provider-delegation.html#using-delegatecomponent))
    is defined for each service. It delegates calling a service method to
    calling that method for a parameter type ('Delegate').  Binding a service
    to an implementation is setting the parameter type ('Delegate') to a type
    that implements the service trait.
  * A `struct` associating 'Delegate' types with implementation (provider)
    types is defined to bind multiple services.  This functions as a Dependency
    Injection module.
  * A value of a generic 'Context' type implementing the traits for requsted
    services serves as an injector.
* [std::error::Request](https://doc.rust-lang.org/std/error/struct.Request.html#method.provide_value),
  and [std::error::Error::provide](https://doc.rust-lang.org/std/error/trait.Error.html#method.provide)
  * A [`request_ref`](https://doc.rust-lang.org/std/error/fn.request_ref.html), and
    a [`request_value`](https://doc.rust-lang.org/std/error/fn.request_value.html)
    function is used to request (inject) a value (service) of a given type, or
    a reference to it.
  * The result is `Option<T>`, or `Option<&'a T>`, since the injector (`Error`)
    is allowed to not be able to provide a given service at a given time.
  * A [`provide` method](https://doc.rust-lang.org/std/error/trait.Error.html#method.provide)
    binds request types to injected values.
  * Each binding is performed by calling
    [`Request::provide_value<T>`](https://doc.rust-lang.org/std/error/struct.Request.html#method.provide_value),
    or
    [`Request::provide_ref<T>`](https://doc.rust-lang.org/std/error/struct.Request.html#method.provide_ref).
  * The current implementation uses
    [`std::any::TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html)
    to produce a (128-bit) unique ID for each type argument passed to
    `request_ref`, or `request_value`.  The ID is used to create a `Request`
    struct which can store and retrieve a value of only the given type
    argument.
    * A call to `Request::provide_value<T>`, or `Request::provide_ref<T>`
      creates a `Request` that stores the value passed to it only if it has a
      matching type ID.
    * `request_ref<'a, T>` and `request_value<'a, T>` call `Error::provide`
      with a `Request` matching only `T`.  Then, `Error::provide` calls
      `request::provide_value`, `request::provide_ref`, or etc. for each value
      that the `Error`-implementing object knows how to provide.  Only a call
      with a matching type ID stores a provided value.  All the other
      `request::provide_*` calls have no effect.  If a value with a matching
      type ID was stored in the `Request` object, `request_ref` or
      `request_value` returns it as `Some(injected_result)`.  If no such value
      was stored, it returns `None`.
* [Pavex](https://github.com/LukeMathWalker/pavex/blob/main/ARCHITECTURE.md)
  uses its own build tool in order to perform code generation before building.  It
  uses the 'classic' style of listing injected services as function
  parameters.  It allows using a service trait in the place of a type in the
  function parameter declaration.  Service bindings are generated from a
  'Blueprint' file, and by analyzing type signatures, annotations (Rust
  procedural macro results) and other output from `rustdoc`.  See
  [Reasoning about Rust: an introduction to Rustdoc's JSON format - Luca Palmieri - EuroRust 2023](https://www.youtube.com/watch?v=OxQYyg_v3rw).
* Other Dependency Injection crates.  Many of them try to bring code to an object-oriented
  approach.

[^1]: The
  [Context-Generic Programming Patterns (DRAFT)](https://patterns.contextgeneric.dev/introduction.html)
  book currently contains deprecated syntax.  See, e.g., the
  [CGP v0.6.0 Release - Major ergonomic improvements for provider and context implementations](https://contextgeneric.dev/blog/v0-6-0-release/)
  post for more recent syntax.

## Design

### Service Interface

```rust
pub trait GetUrl {
    type Url;
    type HttpResponse;
    type Error;

    fn get_url(
        &self,
        url: &Self::Url,
    ) -> impl Future<Output = Result<Self::HttpResponse, Self::Error>>;
}
```

A service would then be represented by a generic type implementing the service
trait (`T: GetUrl`), or by a trait object (e.g., `&impl GetUrl`).

```rust
fn use_generic_type<GetUrlService: GetUrl>(
  get_url_service: &mut GetUrlService
) {
    let result = get_url_service.get_url(&Url::from("https://very-interesting.org"));
    // . . .
}

fn use_trait_object(get_url_service: &mut impl GetUrl) {
    let result = get_url_service.get_url(&Url::from("https://very-interesting.org"));
    // . . .
}
```

### Service Client Design Options

#### Explicit Service Arguments (including Constructor Injection)

```rust
async fn calculate_web_page_message_digest<
    HttpClientService: GetUrl,
    MessageDigestService: NewDigestCalculator,
>(
    http_client_service: &mut HttpClientService,
    message_digest_service: &mut MessageDigestService,
    url: &Url,
) -> Result<Digest, Error>
where
    HttpClientService::HttpResponse: IntoChunkStream,
    MessageDigestService::DigestCalculator:
        IntoDigestOctets<DigestOctets = Digest> + Write,
{
    let mut digest_calculator = message_digest_service.new_digest_calculator()?;
    let mut chunk_stream = http_client_service
        .get_url(url)
        .await?
        .into_chunk_stream();

    while let Some(chunk_result) = chunk_stream.next().await {
        let chunk = chunk_result?;
        digest_calculator.write_all(&chunk)?;
    }

    Ok(digest_calculator.into_digest_octets()?)
}
```

* Keeping track of numerous parameters may get laborious.
* Allows manual binding for each call, or generating code for constructing
  each injected parameter.  E.g.,

  ```rust
  // Generated from the signature of `calculate_web_page_message_digest`:
  async fn calculate_web_page_message_digest_with_injector<
      Injector: InjectGetUrlService + InjectNewDigestCalculatorService,
  >(
      injector: &mut Injector,
      url: &Url,
  ) -> Result<Digest, Error>
  where
      <<Injector as InjectGetUrlService>::Service as HttpClientService>::HttpResponse:
          IntoChunkStream,
      <<Injector as InjectNewDigestCalculatorService>::Service as MessageDigestService>::DigestCalculator:
          IntoDigestOctets<DigestOctets = Digest> + Write,
  {
      let mut http_client_service = InjectGetUrlService::inject(injector);
      let mut message_digest_service = InjectNewDigestCalculatorService::inject(injector);

      calculate_web_page_message_digest(
          &mut http_client_service,
          &mut message_digest_service,
          url,
      )
  }
  ```

### Explicit Injector Argument

This is (essentially)
[the generated `. . . _with_injector` function from Explicit Service Arguments (including Constructor Injection)](#explicit-service-arguments-including-constructor-injection).

```rust
async fn calculate_web_page_message_digest_with_injector<
    'fn_call,
    Injector: InjectRef<'fn_call, HttpClientService> + InjectRef<'fn_call, MessageDigestService>,
>(
    &self,
    injector: &'fn_call Injector,
    url: &Self::Url,
) -> Result<Self::Digest, Self::Error>
where
    HttpClientService: 'fn_call,
    MessageDigestService: 'fn_call,
    &'fn_call Injector: Send
{
    let message_digest_service: &MessageDigestService = InjectRef::inject_ref(injector);
    let mut digest_calculator = message_digest_service.new_digest_calculator()?;
    let http_client_service: &HttpClientService = InjectRef::inject_ref(injector);
    let mut chunk_stream = http_client_service
        .get_url(url)
        .await?
        .into_chunk_stream();

    while let Some(chunk_result) = chunk_stream.next().await {
        let chunk = chunk_result?;
        digest_calculator.write_all(&chunk)?;
    }

    Ok(digest_calculator.into_digest_octets()?)
}
```

* Reduced number of call parameters.
* Injectors can be shared among calls (without repeating each argument
  construction).
* Overriding a specific service requires a new injector.
* Allows accomodating run-time injection features.
  * An injector can change the value it returns for a `Service::inject(injector)`
    call at run time.
  * Defining `inject_fallible(&mut self) -> Result<Self::Service, Error>` can
    accomodate injection that can fail at run time.
  * Similarly, `inject_optional(&mut self) -> Option<Self::Service>` can
    accomodate injecting a service that may not be available at a given time during
    application execution.
* Asynchronous injection can be similarly accomodated.
* The above run-time features can also be implemented by the generated
  `. . . _with_injector` function.

### Explicit All-Services Argument

This is the style that CGP uses.  There's one object of a generic 'Context' type
passed to a client function.  The 'context' object implements all services that the
client requires.

```rust
// [AllServicesObject] is usually named `Context` in CGP.
fn calculate_web_page_message_digest_with_all_services_object<
    AllServicesObject: GetUrl + NewDigestCalculator,
>(
    all_services_object: &mut AllServicesObject,
    url: &Url,
) -> Result<Digest, Error>
where
    <AllServicesObject as GetUrl>::HttpResponse:
        IntoChunkStream,
    <AllServicesObject as NewDigestCalculatorService>::DigestCalculator:
        IntoDigestOctets<DigestOctets = Digest> + Write,
{
    let mut digest_calculator = all_services_object.new_digest_calculator()?;
    let mut chunk_stream = all_services_object
        .get_url(url)
        .await?
        .into_chunk_stream();

    while let Some(chunk_result) = chunk_stream.next().await {
        let chunk = chunk_result?;
        digest_calculator.write_all(&chunk)?;
    }

    Ok(digest_calculator.into_digest_octets()?)
}
```

* Can be added as a trait to `Self`.  This allows adding services to methods,
  and removing them without changing the number of arguments.
* Overriding an individual service requires a new type, possibly forwarding
  all but one services methods to the previous all-services object.
* The services required by a function can still be discovered from the list of
  super-traits in the signature (`GetUrl + NewDigestCalculator`).
* Listing the services required by a function is shorter than having a type
  parameter for each service.
* Changing bindings at run time can still be accomodated by appropriately
  instantiating an all-services object that forwards calls to service objects
  constructed at run time.
* Fallible, and optional injections can be similarly implemented.
* Compared to having a separate argument for each service, this is a trade-off
  between passing less arguments in a function call, and generating less
  template code for accessing each service.

### Using Implementation-Independent Macros

Another approach is to try to express the intent of a service, and a consumer
in a way that allows code for any of the above implementations to be
generated.  This would look more like a Domain-Specific Language processed,
e.g. by macro expansion with some configuration for what code to generate.

```rust
#[injectable]
trait CalculateWebPageMessageDigest {
    type Error;

    #[inject]
    fn http_client_service(&self) -> &mut impl GetUrl;

    #[inject]
    fn message_digest_service(&self) -> &mut impl NewDigestCalculator;

    async fn calculate_web_page_message_digest(
        &self,
        url: &Self::GetUrlService::Url,
    ) -> Result<
        Self::NewDigestCalculatorService::IntoDigestOctets::DigestOctets,
        Self::Error
    > {
        let mut digest_calculator =
            self.message_digest_service().new_digest_calculator()?;
        let mut chunk_stream = self
            .http_client_service()
            .get_url(url)
            .await?
            .into_chunk_stream();

        while let Some(chunk_result) = chunk_stream.next().await {
            let chunk = chunk_result?;
            digest_calculator.write_all(&chunk)?;
        }

        Ok(digest_calculator.into_digest_octets()?)
    }
}
```

Proposed meaning of the syntax in the above example:

* `#[injectable]` declares `CalculateWebPageMessageDigest` as the interface of
  an injectable service (similar to
  [the `@Injectable` decorator in Angular](https://angular.dev/api/core/Injectable)).
  This gives an `injectable` procedural macro the chance to process the trait
  we're declaring.
  * For a trait that contains multiple functions, we can generate per-function
    traits (CGP-style) to allow partial implementation of larger services, as
    well as requiring a partial implementation when injecting.
    * In the above example we're declaring a trait that contains a single
      function, so it wouldn't require generating per-function traits.
* `#[inject]` declares a getter that is auto-generated, and returns an
  injected service.
  * In the above example, we'll also need to generate associated types:

    ```rust
    trait CalculateWebPageMessageDigest {
        // Corresponds to the return type of:
        // #[inject]
        // fn http_client_service(&self) -> &mut impl GetUrl;
        type GetUrlService: GetUrl;

        // Corresponds to the return type of:
        // #[inject]
        // fn message_digest_service(&self) -> &mut impl NewDigestCalculator;
        type NewDigestCalculatorService: NewDigestCalculator;
    }
    ```

## Challenges

### Borrowing Multiple Services Mutably from an Injector

This may require using run-time borrow checking
([Interior Mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)),
or `unsafe` code.

In [§Explicit Injector Argument](#explicit-injector-argument) we had the
`calculate_web_page_message_digest_with_injector` function that obtained two
services by calling `InjectRef::inject_ref(injector)`.

Imagine that we wanted inject the two services as mutable references, e.g.,
because the services have internal state, and we want to allow them to change
it.

Let's say we had the following trait for this purpose:

```rust
/// Allows injecting a mutable reference to a service
///
/// This trait is usually implemented by an injector type.
pub trait InjectMut<'self_lifetime, T> {
    fn inject_mut(&'self_lifetime mut self) -> &'self_lifetime mut T;
}
```

Without interior mutability, we can't return a mutable reference from an
injector, unless the injector object is also borrowed mutably.  So, we need
`&mut self` above.

The following won't work:

```rust
async fn calculate_web_page_message_digest<
    'fn_call,
    Injector: InjectMut<'fn_call, Self::HttpClientService>
        + InjectMut<'fn_call, Self::MessageDigestService>,
>(
    &self,
    injector: &'fn_call mut Injector,
    url: &Self::Url,
) -> Result<Self::Digest, Self::Error>
where
    Self::HttpClientService: 'fn_call,
    Self::MessageDigestService: 'fn_call,
    &'fn_call Injector: Send,
{
    // Throughout the lifetime of `message_digest_service`, `injector` is
    // borrowed mutably.
    let message_digest_service: &mut Self::MessageDigestService =
        InjectMut::inject_mut(injector);

    // We want to use `http_client_service` before `message_digest_service` is
    // out of scope, but it also requires borrowing `injector` mutably.
    let http_client_service: &mut Self::HttpClientService = InjectMut::inject_mut(injector);

    // . . .
}
```

([Crate example](crates/example-of-injector-mutable-borrow-challenge/src/main.rs))

Even though `injector` may be able to provide mutable references to multiple
services safely, expressing that in the Borrow Checker type system is
currently not trivial.

One possibily wat to do that is by having `inject_mut` return both a service
object, and an updated injector, which is able to inject all services, except
the one that was just injected.

### Nameless Return Types

Currently (in 2025) existing library functions, and common idioms, such as
`async` functions, can have return types that cannot be named.  (E.g., see
[Announcing `async fn` and return-position `impl Trait` in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/).)
This can make wrapping existing libraries, or `async` functions as injectable
service traits with a standardized structure that allows, e.g. requesting
additional traits for a return type rather difficult.

* E.g.:

  ```rust
  pub trait GetUrl {
      // We may have a standardized structure for traits that expose a
      // function, in which the function parameter, and return types are
      // accessible:
      type Url;
      type Output;

      // Currently, an `async` function declaration is syntactic sugar for a
      // function with a return type that `impl Future`, but cannot be
      // named.  This gets in the way of writing code that may, or may not
      // require additional constraints, such as `Send` for the returned
      // future. 
      fn get_url(&self, url: &Self::Url) -> Self::Output;
  }

  // There's currently no way to `impl GetUrl for UrlGetService` that looks
  // like the following in stable Rust.
  struct UrlGetService {}

  impl GetUrl for UrlGetService {
      type Url = reqwest::Url;

      // This currently requires unstable Rust:
      type Output = impl Future<Output = Result<reqwest::Response, reqwest::Error>>;

      async fn get_url(
          &self,
          url: &Self::Url
      ) -> Result<Self::HttpResponse, Self::Error> {
          reqwest::get(url.clone()).await
      }
  }
  ```

  * See
    [Async in public trait](https://users.rust-lang.org/t/async-in-public-trait/108400),
    and
    [Crate trait_variant](https://docs.rs/trait-variant/latest/trait_variant/)
    for more details.

### Multi-binding

Assembling a list of service implementations (called
[Multibindings](https://dagger.dev/dev-guide/multibindings.html) in Dager)
is not straightforward because
([macro invocations are not intended to have state](https://github.com/rust-lang/cargo/issues/9084#issuecomment-778687670)).

* Outputting `const` expressions that add to a collection from a macro may work.
* The [inventory crate](https://crates.io/crates/inventory) has such an
  approach, but const evaluation seems to be a challenge.  The
  [linkme crate](https://crates.io/crates/linkme) relies on a linker to
  assemble the result, and doesn't seem to support WebAssemby.
* A
  [crate build script](https://doc.rust-lang.org/cargo/reference/build-script-examples.html)
  is an option, but it may require re-scanning all code to function
  properly, and
  [generating crate dependencies would currently not work](https://doc.rust-lang.org/cargo/reference/build-script-examples.html).
