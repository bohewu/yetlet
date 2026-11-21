use super::{HostFailure, HostOperation, ShutdownHandle};
use crate::composition::{
    Attached, Builder, ContextCallback, Empty, ReleaseCompletion, Resource, ResourceParts,
};
use std::{
    task::{Context, Poll, ready},
    time::Duration,
};

/// An inert, typed descriptor with explicit acquisition and final release.
///
/// Store partial acquisition and in-flight cleanup in self before returning.
/// Pending/retryable Err retain that same owner; arrange a wake for automatic
/// driving or explicitly arrange another caller poll. Released(Err) is terminal.
/// Before acquisition, ordinary Drop must be sufficient. This is not an adoption
/// protocol for already-live asynchronous owners.
pub trait ManagedResource {
    type Error;

    fn poll_acquire(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error>;
}

/// Typed passive dependency storage; inferred by ResourceGroup::with_resource.
pub struct Retained<R, P> {
    pub(super) value: R,
    pub(super) previous: P,
}

/// Typed managed descriptor storage; inferred by with_managed_resource.
pub struct Managed<R, P> {
    pub(super) resource: R,
    pub(super) previous: P,
    pub(super) name: &'static str,
}

mod sealed {
    pub trait Plan {}
}

/// Sealed support for the provided typed dependency lists. Custom Hosts use
/// these lists and implement ManagedResource, never a second lifecycle engine.
#[doc(hidden)]
pub trait ResourcePlan<E>: sealed::Plan {
    type Parts: ResourceParts<HostFailure<E>>;
    fn into_builder(self, host: &'static str) -> Builder<HostFailure<E>, Self::Parts>;
    fn poll_acquire(
        parts: &mut Self::Parts,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>>;
}

impl sealed::Plan for () {}
impl<E> ResourcePlan<E> for () {
    type Parts = Empty;

    fn into_builder(self, _: &'static str) -> Builder<HostFailure<E>> {
        Builder::new()
    }

    fn poll_acquire(
        _: &mut Empty,
        _: &mut Context<'_>,
        _: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>> {
        Poll::Ready(Ok(()))
    }
}

fn release_passive<R, E>(
    _: &mut R,
    _: Duration,
) -> Result<ReleaseCompletion<HostFailure<E>>, HostFailure<E>> {
    Ok(ReleaseCompletion::Released(Ok(())))
}

impl<R, P> sealed::Plan for Retained<R, P> {}
impl<E, R, P: ResourcePlan<E>> ResourcePlan<E> for Retained<R, P> {
    type Parts = Attached<
        P::Parts,
        Resource<
            R,
            fn(&mut R, Duration) -> Result<ReleaseCompletion<HostFailure<E>>, HostFailure<E>>,
        >,
    >;

    fn into_builder(self, host: &'static str) -> Builder<HostFailure<E>, Self::Parts> {
        self.previous.into_builder(host).with_resource(
            "resource",
            self.value,
            release_passive::<R, E> as fn(&mut R, Duration) -> _,
        )
    }

    fn poll_acquire(
        parts: &mut Self::Parts,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>> {
        P::poll_acquire(&mut parts.previous, cx, shutdown)
    }
}

/// Retained acquisition progress. The descriptor is never publicly replaceable.
#[doc(hidden)]
pub struct ResourceSlot<R> {
    descriptor: R,
    attempted: bool,
    acquired: bool,
    host: &'static str,
    name: &'static str,
}

impl<R: ManagedResource> ResourceSlot<R> {
    fn poll_acquire(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), HostFailure<R::Error>>> {
        if self.acquired {
            return Poll::Ready(Ok(()));
        }
        self.attempted = true;
        ready!(self.descriptor.poll_acquire(cx))
            .map_err(|e| HostFailure::resource(self.host, self.name, HostOperation::Acquire, e))?;
        self.acquired = true;
        Poll::Ready(Ok(()))
    }

    fn release(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<HostFailure<R::Error>>, HostFailure<R::Error>> {
        if !self.attempted {
            return Ok(ReleaseCompletion::Released(Ok(())));
        }
        match self.descriptor.poll_release(cx, budget) {
            Ok(ReleaseCompletion::Pending) => Ok(ReleaseCompletion::Pending),
            Ok(ReleaseCompletion::Released(result)) => {
                Ok(ReleaseCompletion::Released(result.map_err(|e| {
                    HostFailure::resource(self.host, self.name, HostOperation::Release, e)
                })))
            }
            Err(e) => Err(HostFailure::resource(
                self.host,
                self.name,
                HostOperation::ReleaseProgress,
                e,
            )),
        }
    }
}

impl<R, P> sealed::Plan for Managed<R, P> {}
impl<E, R: ManagedResource<Error = E>, P: ResourcePlan<E>> ResourcePlan<E> for Managed<R, P> {
    type Parts = Attached<
        P::Parts,
        Resource<
            ResourceSlot<R>,
            ContextCallback<
                fn(
                    &mut ResourceSlot<R>,
                    &mut Context<'_>,
                    Duration,
                ) -> Result<ReleaseCompletion<HostFailure<E>>, HostFailure<E>>,
            >,
        >,
    >;

    fn into_builder(self, host: &'static str) -> Builder<HostFailure<E>, Self::Parts> {
        self.previous.into_builder(host).with_polled_resource(
            self.name,
            ResourceSlot {
                descriptor: self.resource,
                attempted: false,
                acquired: false,
                host,
                name: self.name,
            },
            ResourceSlot::<R>::release as fn(&mut ResourceSlot<R>, &mut Context<'_>, Duration) -> _,
        )
    }

    fn poll_acquire(
        parts: &mut Self::Parts,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>> {
        ready!(P::poll_acquire(&mut parts.previous, cx, shutdown))?;
        if shutdown.is_requested() {
            return Poll::Ready(Ok(()));
        }
        parts
            .last
            .driver_value_mut()
            .expect("resource retained during acquisition")
            .poll_acquire(cx)
    }
}
