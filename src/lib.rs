use tracing_glog::Glog;
use tracing_glog::GlogFields;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

#[derive(Debug, Default, Clone)]
/// Options for log configuration
pub struct LogOpts {
    /// Set the logging level based on the set of filter directives.
    pub log: Vec<Directive>,
    /// Turn off ANSI color codes in the logs
    pub no_color: bool,
    /// Enable tokio-console (if feature enabled)
    pub enable_tokio_console: bool,
}

#[cfg(feature = "clap")]
#[derive(clap::Parser, Debug, Default, Clone)]
/// Options for log configuration
pub struct LogOptsClap {
    /// Set the logging level based on the set of filter directives.
    #[clap(short, long, default_value = "info", global = true)]
    log: Vec<Directive>,

    /// Turn off ANSI color codes in the logs
    #[clap(long, global = true)]
    no_color: bool,

    /// Enable tokio-console
    #[cfg(feature = "console-subscriber")]
    #[clap(long, global = true)]
    enable_tokio_console: bool,
}

#[cfg(feature = "clap")]
impl From<LogOptsClap> for LogOpts {
    fn from(opts: LogOptsClap) -> Self {
        LogOpts {
            log: opts.log,
            no_color: opts.no_color,
            #[cfg(feature = "console-subscriber")]
            enable_tokio_console: opts.enable_tokio_console,
            #[cfg(not(feature = "console-subscriber"))]
            enable_tokio_console: false,
        }
    }
}

#[allow(clippy::needless_doctest_main)]
/// Initialize logging
/// # Example w/ Clap
///
/// ```
/// # #[cfg(feature = "clap")]
/// # {
/// use clap::Parser;
/// use jt_init_logging::{LogOptsClap, init_logging};
///
/// #[derive(Parser)]
/// struct Args {
///     // ... your other args ...
///
///     #[clap(flatten)]
///     log_opts: LogOptsClap,
/// }
///
/// fn main() {
///     // parse args
///     let args = Args::parse();
///     init_logging(&args.log_opts.into());
///     // ... your code ...
/// }
/// # }
/// ```
///
/// # Example w/o Clap
/// ```
/// use jt_init_logging::{LogOpts, init_logging};
///
/// fn main() {
///     init_logging(&LogOpts::default());
///     // ... your code ...
/// }
/// ```
pub fn init_logging(opts: &LogOpts) {
    try_init_logging(opts).expect("to set global subscriber");
}

/// Initialize logging
pub fn try_init_logging(opts: &LogOpts) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let fmt = tracing_subscriber::fmt::Layer::default()
        .with_writer(std::io::stderr)
        .event_format(Glog::default().with_timer(tracing_glog::LocalTime::default()))
        .with_ansi(!opts.no_color)
        .fmt_fields(GlogFields::default())
        .with_filter(
            opts.log
                .iter()
                .cloned()
                .fold(EnvFilter::from_default_env(), |filter, directive| {
                    filter.add_directive(directive)
                }),
        );

    let subscriber = Registry::default().with(fmt);

    #[cfg(feature = "console-subscriber")]
    let tc = opts.enable_tokio_console.then(|| {
        console_subscriber::Builder::default()
            .with_default_env()
            // .server_addr((std::net::Ipv6Addr::UNSPECIFIED, port))
            // Retain just 5m of completed tasks to avoid potentially OOMing.
            .retention(std::time::Duration::from_secs(300))
            .spawn()
    });

    #[cfg(feature = "console-subscriber")]
    let subscriber = subscriber.with(tc);

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
