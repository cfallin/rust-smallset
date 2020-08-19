# SmolSet

[![Crate](https://img.shields.io/crates/v/smolset.svg)](https://crates.io/crates/smolset)

This crate implements a small unordered-set data structure implemented using
[smallvec](https://crates.io/crates/smallvec/).
It initially stores set elements in a simple unordered array.
When the set is smaller than a parameterizable size, no allocations will be performed.
The data structure is thus very space-efficient for sets of only a few elements, much more so than a tree-based or hash-table-based set data structure.
It is also fast when the set is small: queries and inserts perform a linear scan, which is more cache-friendly than a pointer-chasing search through a tree.

However, as the set grows, it will transform internally into a `std::collections::HashSet`.


## Note

This is a fork of the original library here: [rust-smallset](https://github.com/cfallin/rust-smallset).
I have rewritten the internals completely to not have such a bad fallback mode and added more features (and their tests and documentations).
