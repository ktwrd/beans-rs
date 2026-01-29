# Sentry

[Sentry is an error tracking and debugging platform](https://github.com/getsentry/sentry) that can be [self-hosted](https://develop.sentry.dev/self-hosted/) or [rented](https://sentry.io/pricing/).

By default `bean-rs` will send crash dumps to a Sentry instance hosted by `beans-rs` developers, however you can modify `SENTRY_URL` in `src/lib.rs` to send crash dumps to a different Sentry URL address.

By leaving `SENTRY_URL` to it's default value, the crash dumps will be used to help identify critical bugs in `beans-rs`. A small portion of the crash dump is data collected to identify your system and OS version, but most of the data related is to the application itself and it's dependencies.

### Data Collected:

- Machine Hostname
- OS Type and Kernel version
- System Architecture (e.g: x86_64)
- Everything in the log file (which may contain: home directory, network requests, debug information about fltk, etc..)
- What version of the Sentry SDK is being used
- What version of beans-rs you're using
- What version of the rust runtime you're using
- Full stack trace (of rust code) of the error reported