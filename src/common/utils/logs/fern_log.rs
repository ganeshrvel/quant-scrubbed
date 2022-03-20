use crate::common::constants::file_paths::FilePaths;
use crate::common::utils::files::file::create_file;
use fern::colors::{Color, ColoredLevelConfig};
use std::io;

pub fn setup_logging() -> Result<(), fern::InitError> {
    let f = create_file(FilePaths::LOG);

    if f.is_err() {
        paniq!(
            "An error occurred while creating the log file '{}' (P00006)",
            FilePaths::LOG
        );
    }

    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::Magenta)
        .debug(Color::BrightBlack)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    // configure colors for the name of the level.
    // since almost all of them are the same as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::Blue)
        .debug(Color::BrightBlack)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    let base_config = fern::Dispatch::new();

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{date}][{target}][{level}] {message}",
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = record.level(),
                message = message,
            ));
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(FilePaths::LOG)?);

    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{date}][{target}][{level}] {color_line}{message}{color_line}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        .level(log::LevelFilter::Debug)
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;

    Ok(())
}
