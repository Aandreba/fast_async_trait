# Fast Async Trait
Library to implement async traits that aren't reliant on boxed futures, relying instead on the `type_alias_impl_trait` nightly feature for it's base implementation.

## Limitations
- Currently, only one lifetime per async function is allowed
```rust
pub trait AsyncTrait {
    // allowed
    async fn test1 (&self, right: u8);
    // allowed
    async fn test2 (&self, right: &u8);
    // allowed
    async fn test3<'b> (&'b self, right: &'b u8);
    // compiler error
    async fn test4<'b> (&self, right: &'b u8);
}
```
- Async functions with `Self: Rc<Self>` and similar aren't currently supported

## Example
```rust
#![feature(type_alias_impl_trait)]

use fast_async_trait::*;

#[async_trait_def]
pub trait AsyncTrait {
    type Item;

    async fn owned (self) -> Option<Self::Item>;
    async fn by_ref (&self) -> Option<Self::Item>;
    async fn by_mut (&mut self) -> Option<Self::Item>;
    
    #[inline]
    async fn owned_default (self, _n: usize) -> Option<Self::Item> where Self: Sized {
        return self.owned().await
    }

    #[inline]
    async fn by_ref_default (&self, n: usize) -> Option<Self::Item> {
        for _ in 0..n {
            let _ = self.by_ref().await;
        }
        return self.by_ref().await
    }

    #[inline]
    async fn by_mut_default (&mut self, n: usize) -> Option<Self::Item> {
        for _ in 0..n {
            let _ = self.by_mut().await;
        }
        return self.by_mut().await
    }
}

#[async_trait_impl]
impl AsyncTrait for (usize, &[u8]) {
    type Item = u8;

    async fn owned (self) -> Option<Self::Item> {
        return self.1.get(self.0).copied()
    }

    #[inline]
    async fn by_ref (&self) -> Option<Self::Item> {
        return self.1.get(self.0).copied()
    }

    #[inline]
    async fn by_mut (&mut self) -> Option<Self::Item> {
        let v = self.1.get(self.0).copied();
        self.0 += 1;
        return v;
    }
}
```

## Nightly features

The `type_alias_impl_trait` nightly feature is required to be able to add `impl Trait` types (in our case, `impl Future` types) as associated generic types of a trait, which this crate relies on.