#![feature(type_alias_impl_trait, associated_type_defaults, unboxed_closures, fn_traits)]

use fast_async_trait::*;

#[async_trait_def]
pub trait AsyncIterator {
    type Item;

    async fn next (&mut self) -> Option<Self::Item>;
    
    #[inline]
    async fn nth (&mut self, n: usize) -> Option<Self::Item> {
        let iter = (0..n).map(|_| this.next());
        let _ = futures::future::join_all(iter).await;
        return this.next().await
    }
}

/*impl<I: Iterator> AsyncIterator for I {
    type Item = I::Item;
    type Next<'a> = core::future::Ready<Option<I::Item>> where Self: 'a;

    #[inline]
    fn next<'a> (&'a mut self) -> Self::Next<'a> where Self: 'a {
        return core::future::ready(<Self as Iterator>::next(self))
    }
}

pub struct AsyncMap<I, F> {
    iter: I,
    f: F
}

#[async_trait_impl]
impl<I: AsyncIterator, F: FnMut(I::Item) -> Fut, Fut: Future> AsyncIterator for AsyncMap<I, F> {
    type Item = Fut::Output;

    #[inline]
    async fn next (&mut self) -> Option<Self::Item> {
        let v = self.iter.next().await?;
        return Some((self.f)(v).await)
    }
}

#[cfg(test)]
mod tests {
    fn assert_send<T: Send> (_t: &T) {}

    #[test]
    fn test () {
        let fut = [1, 2, 3];
        assert_send(t);
    }
}*/