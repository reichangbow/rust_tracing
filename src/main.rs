use file_rotate::{
    compression::Compression,
    suffix::{AppendTimestamp, FileLimit},
    ContentLimit, FileRotate,
};
use std::fmt;
use tracing_core::{Event, Subscriber};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext, FormattedFields,
};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::fmt::time;
struct MyFormatter;

// MyFormatter
impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        write!(
            &mut writer,
            "{:<5} {} {} [{}({})] ",
            metadata.level(),
            thread_id::get(),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f%Z"),
            metadata.target(),
            metadata.line().unwrap_or_default()
        )?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

#[tokio::main]
pub async fn main() {
    let _guard = init_tracing();
    tracing::debug!("Debug");
    tracing::info!("Info");
    tracing::warn!("Warn");
    tracing::error!("Error");
    drop(_guard);
}
fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let log_writer = FileRotate::new(
        "./Log/log",
        AppendTimestamp::with_format(
            "%y%m%d%H",
            FileLimit::Age(chrono::Duration::days(14)),
            file_rotate::suffix::DateFrom::Now,
        ),
        ContentLimit::Time(file_rotate::TimeFrequency::Hourly),
        Compression::None,
        #[cfg(unix)]
        None,
    );
let log_writer = tracing_appender::rolling::hourly("./Log", "prefix.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(log_writer);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::DEBUG)
        .with_timer(time::SystemTime)    
        .init();
    // .event_format(MyFormatter)
    guard
}
