use crate::{Failure, ShutdownHandle};
use std::{
    cell::{Cell, RefCell},
    io,
    time::Duration,
};
use yetlet::composition::{Attached, Builder, Empty, ReleaseCompletion, Resource, ResourceParts};

/// An inert managed-resource description. Acquire only after installation in
/// the lifecycle root; store partial owners before any later fallible step.
/// Pre-start Drop must require no asynchronous/fallible cleanup.
///
/// Release progresses an original cleanup operation with the supplied budget.
/// Pending/Err retain it. Released(Err) is terminal cleanup with an error.
/// This protocol neither creates an executor nor automatically polls futures.
pub trait ManagedResource {
    fn acquire(&mut self) -> io::Result<()>;
    fn release_within(&mut self, budget: Duration) -> io::Result<ReleaseCompletion<io::Error>>;
}

/// Supplied passive RAII values keep the original A ownership contract.
pub struct Retained<R, Previous> {
    pub(super) _value: R,
    pub(super) _previous: Previous,
}

/// An inert descriptor appended after its earlier dependencies.
pub struct Managed<R, Previous> {
    resource: R,
    previous: Previous,
    label: &'static str,
}

impl<R, P> Managed<R, P> {
    pub(super) fn new(label: &'static str, resource: R, previous: P) -> Self {
        Self {
            resource,
            previous,
            label,
        }
    }
}

mod sealed {
    pub trait Plan {}
}

/// Typed implementation support; custom Hosts return the provided group/list
/// types and do not implement another resource coordinator.
#[doc(hidden)]
pub trait ResourcePlan: sealed::Plan {
    type Parts: ResourceParts<Failure>;
    fn build(self, host: &'static str) -> Builder<Failure, Self::Parts>;
    /// false means cancellation was observed before dependent activity startup.
    fn acquire(parts: &Self::Parts, shutdown: &ShutdownHandle) -> Result<bool, Failure>;
}

impl sealed::Plan for () {}
impl ResourcePlan for () {
    type Parts = Empty;
    fn build(self, _: &'static str) -> Builder<Failure, Empty> {
        Builder::new()
    }
    fn acquire(_: &Empty, shutdown: &ShutdownHandle) -> Result<bool, Failure> {
        Ok(!shutdown.is_requested())
    }
}

fn release_passive<T>(_: &mut T, _: Duration) -> Result<ReleaseCompletion<Failure>, Failure> {
    Ok(ReleaseCompletion::Released(Ok(())))
}

impl<R, P: ResourcePlan> sealed::Plan for Retained<R, P> {}
impl<R, P: ResourcePlan> ResourcePlan for Retained<R, P> {
    type Parts = Attached<
        P::Parts,
        Resource<R, fn(&mut R, Duration) -> Result<ReleaseCompletion<Failure>, Failure>>,
    >;

    fn build(self, host: &'static str) -> Builder<Failure, Self::Parts> {
        self._previous.build(host).with_resource(
            "resource",
            self._value,
            release_passive::<R> as fn(&mut R, Duration) -> _,
        )
    }

    fn acquire(parts: &Self::Parts, shutdown: &ShutdownHandle) -> Result<bool, Failure> {
        P::acquire(parts.previous(), shutdown)
    }
}

#[doc(hidden)]
pub struct ResourceSlot<R> {
    descriptor: RefCell<R>,
    attempted: Cell<bool>,
    host: &'static str,
    label: &'static str,
}

impl<R: ManagedResource> ResourceSlot<R> {
    fn acquire(&self) -> Result<(), Failure> {
        self.attempted.set(true);
        self.descriptor
            .borrow_mut()
            .acquire()
            .map_err(|e| Failure::resource(self.host, self.label, "acquire", e))
    }

    fn release(&mut self, budget: Duration) -> Result<ReleaseCompletion<Failure>, Failure> {
        if !self.attempted.get() {
            return Ok(ReleaseCompletion::Released(Ok(())));
        }
        match self.descriptor.get_mut().release_within(budget) {
            Ok(ReleaseCompletion::Pending) => Ok(ReleaseCompletion::Pending),
            Ok(ReleaseCompletion::Released(result)) => {
                Ok(ReleaseCompletion::Released(result.map_err(|e| {
                    Failure::resource(self.host, self.label, "release", e)
                })))
            }
            Err(error) => Err(Failure::resource(
                self.host,
                self.label,
                "release progress",
                error,
            )),
        }
    }
}

impl<R: ManagedResource, P: ResourcePlan> sealed::Plan for Managed<R, P> {}
impl<R: ManagedResource, P: ResourcePlan> ResourcePlan for Managed<R, P> {
    type Parts = Attached<
        P::Parts,
        Resource<
            ResourceSlot<R>,
            fn(&mut ResourceSlot<R>, Duration) -> Result<ReleaseCompletion<Failure>, Failure>,
        >,
    >;

    fn build(self, host: &'static str) -> Builder<Failure, Self::Parts> {
        self.previous.build(host).with_resource(
            self.label,
            ResourceSlot {
                descriptor: RefCell::new(self.resource),
                attempted: Cell::new(false),
                host,
                label: self.label,
            },
            ResourceSlot::<R>::release as fn(&mut ResourceSlot<R>, Duration) -> _,
        )
    }

    fn acquire(parts: &Self::Parts, shutdown: &ShutdownHandle) -> Result<bool, Failure> {
        if !P::acquire(parts.previous(), shutdown)? || shutdown.is_requested() {
            return Ok(false);
        }
        parts
            .last()
            .value()
            .expect("resource retained during acquisition")
            .acquire()?;
        Ok(!shutdown.is_requested())
    }
}
