# Dependency Injection in Rust

This repository prototypes, and reviews several ways of implementing
[Dependency Injection](https://martinfowler.com/articles/injection.html) in
Rust.

The desired features are described in [Requirements.md](Requirements.md).
They include:

* Injecting a service by type.
* Binding an implementation to a service type.
* Using services in a service implementation (i.e., a service is also a client).
* Binding one implementation to a service for one client, and another
  implementation to the same service for another client. (Service overriding.)
* Preferable features:
  * Circular dependency detection at compile time;
  * Asynchronous injections;
  * Fallible injections (e.g., an injection that can return
    `Result<ServiceObject, Error>`);
  * Changing bindings at run time;
  * Services that may not be bound at a given time during execution;
* (Borrowed from Context-Generic Programming:) defining, and injecting a
  partial service implementation. (E.g., when only some service methods are
  required, and provided.)

Design options, and challenges are reviewed in [Design.md](Design.md).

Prototype code is under [crates/](crates/).

## Build Pre-requisites

* [`cargo`](https://rust-lang.org/tools/install/)

On Ubuntu:

* `apt install gcc libssl-dev pkgconf`

### Development Pre-requisites

* `cargo install cargo-about`
* `cargo install cargo-deny`

### Recommended Tools

* `cargo install cargo-audit`
