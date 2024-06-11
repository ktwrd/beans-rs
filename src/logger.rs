use std::io;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;
use lazy_static::lazy_static;
use log::{LevelFilter, Log, Metadata, Record};

lazy_static! {
    static ref LOGGER: CustomLogger = CustomLogger {
        inner: Mutex::new(None),
    };
}

struct CustomLogger {
    inner: Mutex<Option<CustomLoggerInner>>,
}

impl CustomLogger {
    // Set this `CustomLogger`'s sink and reset the start time.
    fn renew<T: Write + Send + 'static>(&self, sink: T) {
        *self.inner.lock().unwrap() = Some(CustomLoggerInner {
            start: Instant::now(),
            sink: Box::new(sink),
            sentry: sentry_log::SentryLogger::new().filter(|md| match md.level() {
                log::Level::Error => LogFilter::Exception,
                log::Level::Warn => LogFilter::Event,
                log::Level::Info | log::Level::Debug => LogFilter::Breadcrumb,
                log::Level::Trace => LogFilter::Ignore,
            })
        });
    }
}

impl Log for CustomLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(ref mut inner) = *self.inner.lock().unwrap() {
            inner.log(record);
        }
    }

    fn flush(&self) {
        if let Some(ref mut inner) = *self.inner.lock().unwrap() {
            inner.sentry.flush();
        }
    }
}

struct CustomLoggerInner {
    start: Instant,
    sink: Box<dyn Write + Send>,
    sentry: sentry_log::SentryLogger<NoopLogger>
}
use colored::Colorize;
use sentry_log::{LogFilter, NoopLogger};

impl CustomLoggerInner {
    fn log(&mut self, record: &Record) {
        let mut do_print = true;
        unsafe {
            if LOG_FILTER < record.level() {
                do_print = false;
            }
        }
        if do_print
        {
            let now = self.start.elapsed();
            let seconds = now.as_secs();
            let hours = seconds / 3600;
            let minutes = (seconds / 60) % 60;
            let seconds = seconds % 60;
            let milliseconds = now.subsec_nanos() / 1_000_000;

            #[allow(unused_assignments)]
            let mut data = String::new();
            unsafe {
                data = LOG_FORMAT.to_string();
            }
            data = data.replace("#HOURS", &format!("{:02}", hours))
                .replace("#MINUTES", &format!("{:02}", minutes))
                .replace("#SECONDS", &format!("{:02}", seconds))
                .replace("#MILLISECONDS", &format!("{:03}", milliseconds))
                .replace("#THREAD", &format!("{:x}", thread_id::get()))
                .replace("#LEVEL", &format!("{:6}", record.level()))
                .replace("#CONTENT", &format!("{}", record.args()));

            unsafe {
                if LOG_COLOR {
                    data = match record.level() {
                        log::Level::Error => data.red(),
                        log::Level::Warn => data.yellow(),
                        log::Level::Info => data.normal(),
                        log::Level::Debug => data.green(),
                        log::Level::Trace => data.blue(),
                    }.to_string()
                }
            }

            let _ = write!(
                self.sink,
                "{}\n",
                data
            );
        }
        self.sentry.log(&record);
    }
}
pub fn set_filter(filter: LevelFilter)
{
    unsafe {
        LOG_FILTER = filter;
    }
}
static mut LOG_FILTER: LevelFilter = LevelFilter::Trace;
pub static mut LOG_FORMAT: &str = LOG_FORMAT_DEFAULT;
pub static mut LOG_COLOR: bool = true;
pub const LOG_FORMAT_DEFAULT: &str = "[#HOURS:#MINUTES:#SECONDS.#MILLISECONDS] (#THREAD) #LEVEL #CONTENT";
pub const LOG_FORMAT_MINIMAL: &str = "#LEVEL #CONTENT";
pub fn log_to<T: Write + Send + 'static>(sink: T) {
    LOGGER.renew(sink);
    log::set_max_level(LevelFilter::max());
    // The only possible error is if this has been called before
    let _ = log::set_logger(&*LOGGER);
    assert_eq!(log::logger() as *const dyn Log, &*LOGGER as *const dyn Log);
}
pub fn log_to_stdout() {
    log_to(io::stdout());
}