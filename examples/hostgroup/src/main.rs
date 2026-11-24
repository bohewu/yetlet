use std::{
    error::Error,
    io,
    task::{Context, Poll},
    time::Duration,
};

use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{AdmissionOutcome, HostActivity, WorkerCompletion},
};

#[cfg(not(feature = "web"))]
type HostError = std::convert::Infallible;
#[cfg(feature = "web")]
type HostError = yetlet::web::WebError;

struct LocalHost {
    finish_after_job: bool,
}

impl Host for LocalHost {
    type Error = HostError;
    type Activity = LocalActivity;
    type Resources = ();

    fn name(&self) -> &'static str {
        "local"
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity> {
        ResourceGroup::new(LocalActivity {
            finish_after_job: self.finish_after_job,
            finished_job: false,
        })
    }
}

struct LocalActivity {
    finish_after_job: bool,
    finished_job: bool,
}

impl HostActivity for LocalActivity {
    type Error = HostError;

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), HostError>> {
        println!("local: started");
        Poll::Ready(Ok(()))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), HostError>> {
        Poll::Ready(Ok(()))
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, HostError> {
        // Run one job only after every Host in the group is ready.
        let total: u32 = [2, 3, 5].into_iter().sum();
        println!("local: sum = {total}");
        self.finished_job = true;
        Ok(AdmissionOutcome::Opened)
    }

    fn poll_finished(&mut self, _: &mut Context<'_>) -> Poll<()> {
        if self.finish_after_job && self.finished_job {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), HostError> {
        println!("local: stopping");
        Ok(())
    }

    fn poll_join(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> Result<WorkerCompletion<HostError>, HostError> {
        // The local job runs inline and leaves no worker to join.
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}

#[cfg(not(feature = "web"))]
fn main() -> Result<(), Box<dyn Error>> {
    let mut hosts = HostGroup::new()
        .with_host(LocalHost {
            finish_after_job: true,
        })
        .build()?;
    hosts
        .run()
        .map_err(|error| io::Error::other(error.to_string()))?;
    println!("host group: stopped");
    Ok(())
}

#[cfg(feature = "web")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    use std::{future::poll_fn, pin::pin};
    use yetlet::{composition::StartStatus, web::App};

    let web = App::new()
        .get("/health", || async { "ok" })
        .into_host()
        .with_address("127.0.0.1:0");
    let handle = web.handle();
    let mut hosts = HostGroup::new()
        .with_host(LocalHost {
            finish_after_job: false,
        })
        .with_host(web)
        .build()?;
    let stop = hosts.shutdown_handle();
    let mut drive = pin!(async {
        if poll_fn(|cx| hosts.poll_start(cx)).await == StartStatus::Running
            && let Some(address) = handle.local_addr()
        {
            println!("GET http://{address}/health");
        }
        hosts
            .run_async()
            .await
            .map_err(|error| io::Error::other(error.to_string()))
    });

    tokio::select! {
        result = &mut drive => result?,
        signal = tokio::signal::ctrl_c() => {
            // Keep driving the same group until its activities have joined.
            stop.request_shutdown();
            drive.await?;
            signal?;
        }
    }
    println!("host group: stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use yetlet::composition::{JoinStatus, Phase};

    #[test]
    fn a_finished_local_job_stops_and_joins_the_group() {
        let mut hosts = HostGroup::new()
            .with_host(LocalHost {
                finish_after_job: true,
            })
            .build()
            .unwrap();
        assert!(!hosts.outcomes()[0].start_attempted);

        hosts.run().unwrap();

        assert_eq!(hosts.phase(), Phase::Stopped);
        assert_eq!(hosts.outcomes()[0].join, JoinStatus::Succeeded);
    }
}
