//! Batch-run lifecycle presentation via `runemark::ProgressSink` (#530).
//!
//! Owns presentation only — the concurrency algorithm, sitemap sampling,
//! crawl diagnostics, and verdict computation stay in `runners.rs`. This
//! adapter just routes their *display* through a sink that behaves
//! correctly for a real terminal (interactive `indicatif` bar), a redirected
//! stream (bounded start/notice/finish lines on stderr), or silence
//! (`--quiet` / `--progress never`).

use runemark::{
    Console, ProgressMode, ProgressSink, SilentProgress, TerminalProgress, Tone, Verdict,
};

use auditmysite::cli::ProgressPolicy;
use auditmysite::util::truncate_url;

pub struct BatchLifecyclePresenter {
    sink: Box<dyn ProgressSink>,
}

impl BatchLifecyclePresenter {
    /// Builds the presenter for a real batch run. `--quiet` always wins over
    /// `--progress`, matching the existing `--quiet` contract.
    pub fn new(progress: ProgressPolicy, quiet: bool, console: Console, is_terminal: bool) -> Self {
        let sink: Box<dyn ProgressSink> = if quiet {
            Box::new(SilentProgress)
        } else {
            match progress {
                ProgressPolicy::Never => Box::new(SilentProgress),
                ProgressPolicy::Auto => Box::new(TerminalProgress::stderr(
                    ProgressMode::Auto,
                    console,
                    is_terminal,
                )),
                ProgressPolicy::Always => Box::new(TerminalProgress::stderr(
                    ProgressMode::Always,
                    console,
                    is_terminal,
                )),
            }
        };
        Self { sink }
    }

    /// Builds a presenter around an explicit sink — used by tests to inject
    /// a `PlainProgress<Vec<u8>>` and assert on its exact output.
    #[cfg(test)]
    fn from_sink(sink: Box<dyn ProgressSink>) -> Self {
        Self { sink }
    }

    pub fn start(&self, total: usize, stage: &str) {
        self.sink.start(total as u64, stage);
    }

    /// Advances to `completed` with the (truncated) URL just finished.
    pub fn advance(&self, completed: usize, url: &str) {
        self.sink.advance(completed as u64, &truncate_url(url, 50));
    }

    /// Surfaces a per-URL audit error without corrupting an active bar.
    pub fn notice_error(&self, url: &str, err: &str) {
        self.sink.notice(Tone::Error, &format!("{url}: {err}"));
    }

    /// Surfaces a general lifecycle notice (e.g. a diagnostics one-liner).
    pub fn notice(&self, message: &str) {
        self.sink.notice(Tone::Info, message);
    }

    pub fn finish(&self, verdict: Verdict, message: &str) {
        self.sink.finish(verdict, message);
    }

    /// Closes the lifecycle with AuditMySite's own batch verdict, mapped to
    /// Runemark's semantic outcome.
    pub fn finish_verdict(&self, verdict: auditmysite::Verdict, message: &str) {
        let mapped = match verdict {
            auditmysite::Verdict::Pass => Verdict::Passed,
            auditmysite::Verdict::Warn => Verdict::Warning,
            auditmysite::Verdict::Fail => Verdict::Failed,
        };
        self.finish(mapped, message);
    }

    /// Closes the lifecycle, *then* renders the final report (#580). The
    /// order is fixed here rather than at each call site: rendering first
    /// lets an interactive bar redraw across report lines and makes a
    /// redirected stream print its verdict after the report.
    pub fn finish_then_render<T>(
        &self,
        verdict: auditmysite::Verdict,
        message: &str,
        render: impl FnOnce() -> T,
    ) -> T {
        self.finish_verdict(verdict, message);
        render()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runemark::{ColorMode, PlainProgress};
    use std::sync::{Arc, Mutex};

    /// A `Vec<u8>`-backed sink shared between the presenter and the test so
    /// the buffer can be inspected after the presenter (and its internal
    /// `Mutex`-wrapped writer) has been dropped.
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn plain_presenter() -> (BatchLifecyclePresenter, Arc<Mutex<Vec<u8>>>) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let console = Console::stdout(ColorMode::Never);
        let sink = PlainProgress::new(console, SharedBuf(buf.clone()));
        (BatchLifecyclePresenter::from_sink(Box::new(sink)), buf)
    }

    fn buf_to_string(buf: &Arc<Mutex<Vec<u8>>>) -> String {
        String::from_utf8(buf.lock().unwrap().clone()).unwrap()
    }

    #[test]
    fn start_notice_finish_appear_in_order_and_bounded() {
        let (presenter, buf) = plain_presenter();

        presenter.start(3, "Auditing URLs");
        presenter.advance(1, "https://example.com/a");
        presenter.notice_error("https://example.com/b", "timeout");
        presenter.finish(Verdict::Warning, "2/3 passed");

        let output = buf_to_string(&buf);
        let lines: Vec<&str> = output.lines().collect();
        // start + notice + finish = 3 lines; advance() is a documented no-op
        // for the plain sink (bounded output — no per-URL success line).
        assert_eq!(lines.len(), 3, "unexpected output: {output:?}");
        assert!(lines[0].contains("Auditing URLs") && lines[0].contains("(0/3)"));
        assert!(lines[1].contains("timeout"));
        assert!(lines[2].contains("2/3 passed"));
    }

    #[test]
    fn advance_does_not_produce_one_line_per_url() {
        let (presenter, buf) = plain_presenter();
        presenter.start(5, "Auditing URLs");
        for i in 1..=5 {
            presenter.advance(i, &format!("https://example.com/{i}"));
        }
        presenter.finish(Verdict::Passed, "done");

        let output = buf_to_string(&buf);
        assert_eq!(
            output.lines().count(),
            2,
            "advance() must stay silent: {output:?}"
        );
    }

    #[test]
    fn lifecycle_is_finished_before_report_renders() {
        let (presenter, buf) = plain_presenter();
        presenter.start(2, "Auditing URLs");

        let seen_at_render =
            presenter.finish_then_render(auditmysite::Verdict::Warn, "1/2 passed", || {
                buf_to_string(&buf)
            });

        assert!(
            seen_at_render
                .lines()
                .last()
                .is_some_and(|l| l.contains("1/2 passed")),
            "verdict must be written before the report renders: {seen_at_render:?}"
        );
        assert_eq!(buf_to_string(&buf), seen_at_render);
    }

    #[test]
    fn quiet_forces_silence_regardless_of_progress_policy() {
        let presenter = BatchLifecyclePresenter::new(
            ProgressPolicy::Always,
            true,
            Console::stdout(ColorMode::Never),
            false,
        );
        // SilentProgress never touches any writer — calling every method must
        // not panic and there is nothing to assert on beyond that.
        presenter.start(1, "x");
        presenter.advance(1, "https://example.com");
        presenter.notice_error("https://example.com", "boom");
        presenter.finish(Verdict::Failed, "done");
    }

    #[test]
    fn progress_never_forces_silence() {
        let presenter = BatchLifecyclePresenter::new(
            ProgressPolicy::Never,
            false,
            Console::stdout(ColorMode::Never),
            false,
        );
        presenter.start(1, "x");
        presenter.finish(Verdict::Passed, "done");
    }
}
