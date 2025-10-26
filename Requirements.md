# Depedency Injection Requirements

What follows is roughly equivalent to a Requirements Document.  Different
designs are considered in [Design.md](Design.md).

## Desired Features

* Injecting a service by type.
  * In Rust the analogue of the
    [Newtype Pattern](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)
    for traits can usually replace the need for techniques like
    [`@Named`](https://docs.oracle.com/javaee/7/api/javax/inject/Named.html),
    and other qualifiers in Java Enterprise Edition (called
    [keyed services](https://learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection#keyed-services)
    in .NET).
* Binding an implementation to a service type.
* Using injected services in a service implementation (i.e., a service is also
  a client).
* Binding one implementation to a service for one client, and another
  implementation to the same service for another client. (Service overriding.)
* A service can have internal state.
* `async` service methods.  (Currently
  [non-trivial](https://users.rust-lang.org/t/async-in-public-trait/108400).)
* Preferably, circular dependency detection at compile time.
* Here we prefer Dependency Injection more appropritate for compile-time
  resolution.
  I.e., the following are additional features, though still preferable.
  * Asynchronous injections;
  * Fallible injections (e.g., an injection that can return
    `Result<ServiceObject, Error>`);
  * Changing bindings at run time;
  * Services that may not be bound at a given time during execution;
* Another preferable feature borrowed from Context-Generic Programming is
  defining, and injecting partial services.  I.e., instead of defining a
  (relatively large) set of methods that each service implementation has to
  have, an implementation can provide individual methods.  A client can also
  specify the individual methods it requires.
  * This also allows implementing, and injecting methods available only on
    specific platforms, without other techniques, such as a separate function
    for checking whether a platform supports a service method, or injecting a
    `Result`, or an `Option` only for the purpose of identifying whether a
    function is implemented, or panicking with a 'not implemented' error.
  * For methods whose availability can be detected only at run time, the above
    approaches may still be necessary.
* Ability to wrap existing functions returning a nameless type (such as a
  Return-Position `impl Trait` type).  (Currently encompasses supporting
  `async` service methods.  See, e.g.
  [Announcing `async fn` and return-position `impl Trait` in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/).)
* Automatic collection of service implementations for a given service
  interface into a collection (called
  [Multibindings](https://dagger.dev/dev-guide/multibindings.html) in Dagger.)

## User Journeys

### Dependency Injection in New Code

Not much to add here.  New code can adapt to design requirements.

### Wrapping an Existing Library as in Injectable Service

#### Adding an Injector to an Existing Library Method

When wrapping a library as an injectable service, we may need to an injector
an existing library method, and propagating the injector to results returned
from the method.

See
[Pattern for Associating CGP Context with Objects Returned by Existing APIs](https://gist.github.com/pavpen/c52e5b2ac2307b115c947ce0cc523d14)
for a more detailed example.

##### Example

Imagine that we want to wrap the following example library as an injectable
service, which reads the `timeout` argument for
`read_device_string_languages` from another injectable service
`UsbIoConfigurationService`.

###### Example `os_usb` Library

```rust
mod os_usb {
    /// Book-keeping information for making calls to the platform
    pub struct LibraryContext {}
    
    impl LibraryContext {
        /// Returns a list of connected USB devices matching at least one
        /// `or_filter`, and to which the current application has been granted
        /// access
        fn request_usb_devices(
            &self,
            or_filters: &[&Self::UsbDeviceFilter],
        ) -> Result<Vec<Self::UsbDeviceObject>, Self::Error> {
            // . . . implementation . . .            
        }
    }

    /// Allows inspecting basic device information, and initiating
    /// communication with  a device
    pub struct UsbDevice {
        pub fn open(&mut self) -> Result<OpenUsbDevice, Error> {
            // . . . implementation . . .
        }
    }

    /// Allows exchanging packets with device endpoints
    struct OpenUsbDevice {
        /// Receives the list of languages in which device strings can be read
        /// from the device
        pub fn read_device_string_languages(
            &self,
            timeout: Duration,
        ) -> Result<Vec<DeviceStringLanguage>, Error> {
            // . . . implementation . . .
        }
    }
}
```

###### Example `UsbIoConfigurationService`

```rust
pub trait UsbIoConfigurationService {
    fn read_device_string_languages_timeout(&mut self) -> Duration;
}
```

###### Example Wrapper Service Interface

```rust
// Mostly corresponds to [os_usb::LibraryContext].
pub trait UsbService {
    type UsbDeviceObject: UsbDevice;
    type Error;

    fn request_usb_devices(
        &self,
        or_filters: &[&Self::UsbDeviceFilter],
    ) -> Result<Vec<Self::UsbDeviceObject>, Self::Error>;
}

pub trait UsbDevice {
    type OpenUsbDeviceObject: OpenUsbDevice;
    type Error;
    
    fn open(&mut self) -> Result<Self::OpenUsbDeviceObject, Self::Error>;
}

pub trait OpenUsbDevice {
    type DeviceStringLanguage;
    type Error;

    // Doesn't have a `timeout` parameter.  In the implementation of this
    // service the timeout should be obtained from an injected
    // `UsbIoConfigurationService`.
    pub fn read_device_string_languages(
        &self,
    ) -> Result<Vec<Self::DeviceStringLanguage>, Self::Error>;
}
```

###### Example `UsbService` Implementation

```rust
/// Allows injecting a `UsbIoConfigurationService` object
pub trait InjectUsbIoConfigurationService {
    type UsbIoConfiguration: UsbIoConfigurationService;

    fn inject(&self) -> Self::UsbIoConfiguration;
}

pub struct OsUsbUsbService<UsbIoConfiguration: UsbIoConfigurationService> {
    // . . .
}

impl<UsbIoConfiguration: UsbIoConfigurationService>
OsUsbUsbService<UsbIoConfiguration> {
    pub fn with_services(UsbIoConfiguration usb_io_configuration) -> Self {
        // . . . implementation . . .
    }
    
    pub fn with_injector<Injector: InjectUsbIoConfigurationService>(
        injector: Injector
    ) -> Self {
        usb_io_configuration = InjectUsbIoConfigurationService::inject(injector);
        
        OsUsbUsbService::with_services(usb_io_configuration)
    }
    
    fn usb_io_configuration_ref(&self) -> &UsbIoConfiguration {
        // . . . implementation . . .
    }
}

impl<UsbIoConfiguration: UsbIoConfigurationService> UsbService for
OsUsbUsbService<UsbIoConfiguration> {
    type UsbDeviceObject = OsUsbUsbDeviceObject;
    type Error = os_usb::Error;

    // Must propagate `self.usb_io_configuration_ref()` to any returned
    // [Self::UsbDeviceObject].
    fn request_usb_devices(
        &self,
        or_filters: &[&Self::UsbDeviceFilter],
    ) -> Result<Vec<Self::UsbDeviceObject>, Self::Error> {
        // . . . implementation . . .
    }
}

// Similarly, [OsUsbUsbDevice] must propagate `&UsbIoConfiguration` to
// [OsUsbOpenUsbDevice], so it can propagate it to:
impl<UsbIoConfiguration: UsbIoConfigurationService> OpenUsbDevice for
OsUsbOpenUsbDevice<UsbIoConfiguration> {
    pub fn read_device_string_languages(
        &self,
    ) -> Result<Vec<Self::DeviceStringLanguage>, Self::Error> {
        let mut usb_io_configuration_service = self.usb_io_configuration_service_mut();
        let timeout = usb_io_configuration_service.control_transfer_in_timeout();

        self.library_open_usb_device().read_device_string_languages(timeout).map(
            |language_list|
            language_list.map(Self::DeviceStringLanguage::from)
        )
    }
}
```

#### Observations

In the above example:

* Each type in the wrapped `os_usb` library must be wrapped in another type
  that adds the services that need to be propagated to to any methods of the
  wrapped type, or to any methods of any results returned by methods of the
  wrapped type (transitively to the full depth of the type graph).
  * The complexity at which this boilerplate code increases is undesirable.
* In the above example, there's also a type parameter for each service that
  must be propagated.
* The increase of service parameters can be contained by passing only one
  injector parameter that can provide all necessary services.
* A similar limit can be achieved by passing an object implementing all
  necessary services like in the Context-Generic Programming style.  (See
  [§Existing Approaches in Design.md](Design.md#existing-approaches)).
