pub trait Visitor<T> {
  type Error;

  fn visit(&self, value: &mut T) -> Result<(), Self::Error>;
}
