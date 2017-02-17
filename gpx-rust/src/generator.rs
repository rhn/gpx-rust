//! Generator convenience functions

extern crate fringe;

use self::fringe::{OsStack, Generator};
use self::fringe::generator::Yielder;

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
pub fn make_gen<'a, F, Output: 'a + Send>(body: F)
        -> Generator<(), Output, OsStack>
        where F: FnOnce(&mut Yielder<(), Output>) + Send + 'a {
    let stack = OsStack::new(1 << 24).unwrap();
    Generator::new(stack, move |y, ()| {body(y)})
}
