#![feature(type_alias_impl_trait, associated_type_defaults)]

use fast_async_trait::*;

#[async_trait_def]
pub trait AsyncIterator {
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

/*type AsyncIteratorAdderDefault<'a, This: 'a + ?Sized + AsyncIterator> = impl 'a + ::core::future::Future;

pub trait AsyncIterator {
    type Item;
    type Next<'__self__>: '__self__ + ::core::future::Future<Output = Option<Self::Item>> where Self: '__self__;
    type Nth<'__self__>: 
        '__self__ + ::core::future::Future<Output = <AsyncIteratorAdderDefault<'__self__, Self> as ::core::future::Future>::Output> = AsyncIteratorAdderDefault<'__self__, Self>
        where Self: '__self__;

    fn next<'__self__>(&'__self__ mut self) -> Self::Next<'__self__> where Self: '__self__;

    #[inline]
    fn nth<'__self__>(
        &'__self__ mut self,
        n: usize,
    ) -> AsyncIteratorAdderDefault<'__self__, Self>
    where
        Self: '__self__,
    {
        return async move {
            for _ in 0..n {
                let _ = self.next().await;
            }
            return self.next().await;
        }
    }
}*/

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