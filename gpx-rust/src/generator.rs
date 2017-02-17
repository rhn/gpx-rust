//! Generator convenience functions

extern crate fringe;

use self::fringe::generator::Yielder;

/// Generator type which passes values from the inside to outside only
pub type Generator<T> = fringe::Generator<(), T, fringe::OsStack>;

/// Creates a new generator
///
/// # Examples
/// ```
/// let gen = make_gen(|y| {
///     y.suspend(Some(2));
/// });
/// for i in gen {
///     println!("{:?}", i);
/// }
pub fn make_gen<'a, F, Output: 'a + Send>(body: F) -> Generator<Output>
        where F: FnOnce(&mut Yielder<(), Output>) + Send + 'a {
    let stack = fringe::OsStack::new(1 << 24).unwrap();
    Generator::new(stack, move |y, ()| {body(y)})
}
