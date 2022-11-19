## Nightly features

The `type_alias_impl_trait` nightly feature is required to be able to add `impl Trait` types (in our case, `impl Future` types) as associated generic types of a trait, which this crate relies on.

The `associated_type_defaults`, `unboxed_closures`, `fn_traits` and `associated_types` nightly features are required to be able to add default async implementations of trait methods.

**Unexpanded**
```rust
#![feature(type_alias_impl_trait, associated_type_defaults, unboxed_closures)]

```

**Expanded**
```rust
```