## Nightly features

The `type_alias_impl_trait` nightly feature is required to be able to add `impl Trait` types (in our case, `impl Future` types) as associated generic types of a trait, which this crate relies on.

The `return_position_impl_trait_in_trait` nightly feature id required to be able to add default async implementations of trait methods.

**Unexpanded**
```rust
#![feature(type_alias_impl_trait, return_position_impl_trait_in_trait)]

```

**Expanded**
```rust
```