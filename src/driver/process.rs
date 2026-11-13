use std::io::Read;
use std::process::{Child, Command, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(super) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

const TIMEOUT_ENV: &str = "V_RUST_TEST_TIMEOUT_SECS";
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub(super) enum RunOutcome {
    Finished(Output),
    /// The child was killed at the deadline; output captured so far is kept.
    TimedOut(Output),
}

/// Execution timeout from `V_RUST_TEST_TIMEOUT_SECS`, defaulting to 10 seconds.
pub(super) fn execution_timeout() -> Duration {
    timeout_from(std::env::var(TIMEOUT_ENV).ok().as_deref())
}

fn timeout_from(value: Option<&str>) -> Duration {
    value
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map_or(DEFAULT_TIMEOUT, Duration::from_secs)
}

/// Runs `cmd` with piped output, killing and reaping it after `timeout`.
pub(super) fn run_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> std::io::Result<RunOutcome> {
    let mut child =
        cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());

    let deadline = Instant::now() + timeout;
    let (status, timed_out) = wait_until(&mut child, deadline)?;
    let output = Output { status, stdout: join(stdout), stderr: join(stderr) };
    Ok(if timed_out { RunOutcome::TimedOut(output) } else { RunOutcome::Finished(output) })
}

fn wait_until(
    child: &mut Child,
    deadline: Instant,
) -> std::io::Result<(std::process::ExitStatus, bool)> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok((status, false));
        }
        if Instant::now() >= deadline {
            // Ignore kill errors: the child may have exited since the last poll.
            let _ = child.kill();
            return Ok((child.wait()?, true));
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut pipe) = pipe {
            // A read error just ends capture; partial output is still useful.
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    })
}

fn join(handle: JoinHandle<Vec<u8>>) -> Vec<u8> {
    handle.join().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::{Duration, Instant};

    #[cfg(windows)]
    fn sleeping_command() -> Command {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"]);
        cmd
    }

    #[cfg(not(windows))]
    fn sleeping_command() -> Command {
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        cmd
    }

    #[test]
    fn kills_child_that_outlives_the_deadline() {
        let start = Instant::now();
        let outcome = run_with_timeout(&mut sleeping_command(), Duration::from_millis(500))
            .expect("sleeping child should spawn");
        assert!(matches!(outcome, RunOutcome::TimedOut(_)));
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn returns_output_of_child_that_finishes_in_time() {
        let mut cmd = Command::new(std::env::current_exe().expect("test exe path"));
        cmd.arg("--list");
        let outcome = run_with_timeout(&mut cmd, Duration::from_secs(30)).expect("should spawn");
        match outcome {
            RunOutcome::Finished(output) => assert!(output.status.success()),
            RunOutcome::TimedOut(_) => panic!("child should not time out"),
        }
    }

    #[test]
    fn parses_timeout_override() {
        assert_eq!(timeout_from(Some("3")), Duration::from_secs(3));
        assert_eq!(timeout_from(Some("abc")), DEFAULT_TIMEOUT);
        assert_eq!(timeout_from(Some("0")), DEFAULT_TIMEOUT);
        assert_eq!(timeout_from(None), DEFAULT_TIMEOUT);
    }
}
