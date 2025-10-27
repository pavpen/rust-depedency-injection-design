pub trait InjectRef<'self_lifetime, T> {
    fn inject_ref(&'self_lifetime self) -> &'self_lifetime T;
}
