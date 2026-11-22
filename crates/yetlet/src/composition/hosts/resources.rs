use super::driver::{DriverPlan, SelectedDriver};
use super::{BlockingDriver, HostFailure, HostOperation, HostRunMode, NoDriver, ShutdownHandle};
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

    /// Validate/configure the selected driver without acquiring resources.
    /// Every group descriptor prepares before any acquisition. Ordinary Drop
    /// must remain sufficient; this hook must not create cleanup obligations.
    fn prepare_driver(&mut self, _mode: HostRunMode) -> Result<(), Self::Error> {
        Ok(())
    }

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

/// Typed driver metadata carried alongside an otherwise unchanged resource plan.
///
/// Use ResourceGroup::with_blocking_driver. This descriptor is inert, and the
/// selected driver is retained separately from activity/resource finalization.
pub struct Driven<D, P> {
    pub(super) driver: D,
    pub(super) previous: P,
}

mod sealed {
    pub trait Plan {}
}

/// Sealed support for the provided typed dependency lists. Custom Hosts use
/// these lists and implement ManagedResource, never a second lifecycle engine.
#[doc(hidden)]
pub trait ResourcePlan<E>: sealed::Plan {
    type Parts: ResourceParts<HostFailure<E>>;
    type Driver: DriverPlan<E>;
    fn into_builder(
        self,
        host: &'static str,
    ) -> (Builder<HostFailure<E>, Self::Parts>, Self::Driver);
    fn prepare_driver(parts: &mut Self::Parts, mode: HostRunMode) -> Result<(), HostFailure<E>>;
    fn poll_acquire(
        parts: &mut Self::Parts,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>>;
}

impl sealed::Plan for () {}
impl<E> ResourcePlan<E> for () {
    type Parts = Empty;
    type Driver = NoDriver;

    fn into_builder(self, _: &'static str) -> (Builder<HostFailure<E>>, NoDriver) {
        (Builder::new(), NoDriver)
    }

    fn prepare_driver(_: &mut Empty, _: HostRunMode) -> Result<(), HostFailure<E>> {
        Ok(())
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

    type Driver = P::Driver;

    fn into_builder(
        self,
        host: &'static str,
    ) -> (Builder<HostFailure<E>, Self::Parts>, Self::Driver) {
        let (previous, driver) = self.previous.into_builder(host);
        (
            previous.with_resource(
                "resource",
                self.value,
                release_passive::<R, E> as fn(&mut R, Duration) -> _,
            ),
            driver,
        )
    }

    fn prepare_driver(parts: &mut Self::Parts, mode: HostRunMode) -> Result<(), HostFailure<E>> {
        P::prepare_driver(&mut parts.previous, mode)
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
    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), HostFailure<R::Error>> {
        self.descriptor.prepare_driver(mode).map_err(|e| {
            HostFailure::resource(self.host, self.name, HostOperation::PrepareDriver, e)
        })
    }

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

    type Driver = P::Driver;

    fn into_builder(
        self,
        host: &'static str,
    ) -> (Builder<HostFailure<E>, Self::Parts>, Self::Driver) {
        let (previous, driver) = self.previous.into_builder(host);
        (
            previous.with_polled_resource(
                self.name,
                ResourceSlot {
                    descriptor: self.resource,
                    attempted: false,
                    acquired: false,
                    host,
                    name: self.name,
                },
                ResourceSlot::<R>::release
                    as fn(&mut ResourceSlot<R>, &mut Context<'_>, Duration) -> _,
            ),
            driver,
        )
    }

    fn prepare_driver(parts: &mut Self::Parts, mode: HostRunMode) -> Result<(), HostFailure<E>> {
        P::prepare_driver(&mut parts.previous, mode)?;
        parts
            .last
            .driver_value_mut()
            .expect("resource retained before acquisition")
            .prepare_driver(mode)
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

impl<D, P> sealed::Plan for Driven<D, P> {}
impl<E, D: BlockingDriver<E>, P: ResourcePlan<E, Driver = NoDriver>> ResourcePlan<E>
    for Driven<D, P>
{
    type Parts = P::Parts;
    type Driver = SelectedDriver<D>;

    fn into_builder(
        self,
        host: &'static str,
    ) -> (Builder<HostFailure<E>, Self::Parts>, Self::Driver) {
        let (builder, NoDriver) = self.previous.into_builder(host);
        (
            builder,
            SelectedDriver {
                driver: self.driver,
                host,
            },
        )
    }

    fn prepare_driver(parts: &mut Self::Parts, mode: HostRunMode) -> Result<(), HostFailure<E>> {
        P::prepare_driver(parts, mode)
    }

    fn poll_acquire(
        parts: &mut Self::Parts,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>> {
        P::poll_acquire(parts, cx, shutdown)
    }
}
