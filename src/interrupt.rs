// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Cooperative interruption: release readers and reap parser children before deleting staged data.

use std::io::{self, Read};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static REQUESTED: AtomicBool = AtomicBool::new(false);
static INSTALLED: OnceLock<Result<(), String>> = OnceLock::new();

pub(crate) fn install() -> io::Result<()> {
    INSTALLED
        .get_or_init(|| {
            ctrlc::set_handler(|| REQUESTED.store(true, Ordering::Relaxed))
                .map_err(|error| error.to_string())
        })
        .clone()
        .map_err(io::Error::other)
}

pub(crate) fn requested() -> bool {
    REQUESTED.load(Ordering::Relaxed)
}

pub(crate) fn check() -> io::Result<()> {
    if requested() {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "Cancelled by Ctrl+C",
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn output(command: &mut Command) -> io::Result<Output> {
    if INSTALLED.get().is_none() {
        return command.output();
    }
    output_until(command, requested)
}

fn output_until(command: &mut Command, cancelled: impl Fn() -> bool) -> io::Result<Output> {
    if cancelled() {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "Cancelled by Ctrl+C",
        ));
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    // Drain both pipes while waiting: a verbose parser must not block on a full pipe.
    std::thread::scope(|scope| {
        let read = |mut stream: Box<dyn Read + Send>| -> io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            stream.read_to_end(&mut bytes)?;
            Ok(bytes)
        };
        let stdout = scope.spawn(move || read(Box::new(stdout)));
        let stderr = scope.spawn(move || read(Box::new(stderr)));
        let status = (|| loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status);
            }
            if cancelled() {
                // Kill and wait even when only the parent received SIGINT. Windows cannot
                // remove a workspace while a child still holds its files open.
                let _ = child.kill();
                return child.wait();
            }
            std::thread::sleep(Duration::from_millis(20));
        })();
        if status.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let join = |result: std::thread::Result<io::Result<Vec<u8>>>| {
            result.map_err(|_| io::Error::other("Parser output reader panicked"))?
        };
        Ok(Output {
            status: status?,
            stdout: join(stdout.join())?,
            stderr: join(stderr.join())?,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_waits_for_cancellation() {
        if let Some(marker) = std::env::var_os("JUNTEI_TEST_INTERRUPT_CHILD") {
            // Write more than a pipe buffer to both pipes before waiting.
            use std::io::Write;
            std::io::stdout()
                .write_all(&vec![b'o'; 128 * 1024])
                .unwrap();
            std::io::stderr()
                .write_all(&vec![b'e'; 128 * 1024])
                .unwrap();
            std::fs::write(marker, b"ready").unwrap();
            std::thread::sleep(Duration::from_secs(60));
        }
    }

    #[test]
    fn cancelling_a_parser_drains_output_and_waits_for_it_to_exit() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("ready");
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "interrupt::tests::child_waits_for_cancellation",
                "--nocapture",
            ])
            .env("JUNTEI_TEST_INTERRUPT_CHILD", &marker);
        let start = std::time::Instant::now();
        let result = output_until(&mut command, || {
            marker.exists() || start.elapsed() > Duration::from_secs(5)
        })
        .unwrap();
        assert!(!result.status.success());
        assert!(marker.exists());
        assert!(result.stdout.len() >= 128 * 1024);
        assert!(result.stderr.len() >= 128 * 1024);
        assert!(start.elapsed() < Duration::from_secs(10));
    }
}
