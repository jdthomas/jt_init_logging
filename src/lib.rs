use tracing_glog::Glog;
use tracing_glog::GlogFields;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

#[derive(clap::Parser, Debug, Default, Clone)]
pub struct LogOpts {
    /// Set the logging level based on the set of filter directives.
    #[clap(short, long, default_value = "info", global = true)]
    pub log: Vec<Directive>,

    /// Turn off ANSI color codes in the logs
    #[clap(long, global = true)]
    pub no_color: bool,

    /// Enable tokio-console
    #[clap(long, global = true)]
    pub enable_tokio_console: bool,
}

pub fn init_logging(opts: &LogOpts) {
    try_init_logging(opts).expect("to set global subscriber");
}

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

    let tc = opts.enable_tokio_console.then(|| {
        console_subscriber::Builder::default()
            .with_default_env()
            // .server_addr((std::net::Ipv6Addr::UNSPECIFIED, port))
            // Retain just 5m of completed tasks to avoid potentially OOMing.
            .retention(std::time::Duration::from_secs(300))
            .spawn()
    });

    let subscriber = Registry::default().with(fmt).with(tc);

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
