# Rust-SLIP
An implementation of SLIP in Rust.

## Notes
Currently uses std; `Vec` and `String` require dyanmic allocation.

To make this no_std, there are a few approaches:

- use the `heapless` crate
- use the `alloc` create and hook up with a global allocator
- use the `ArrayVec` crate (allocates on the stack)