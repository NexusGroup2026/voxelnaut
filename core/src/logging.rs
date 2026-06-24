//! Logging system for VoxelNaut
//!
//! Structured logging with file output and console output.

use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use chrono::Local;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// Log entry
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, target: &str, message: String) -> Self {
        Self {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            level,
            target: target.to_string(),
            message,
        }
    }
}

/// Logger struct
pub struct Logger {
    target: String,
    file: Option<Mutex<BufWriter<File>>>,
    level: LogLevel,
    start_time: Instant,
}

impl Logger {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            file: None,
            level: LogLevel::Info,
            start_time: Instant::now(),
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_file(mut self, path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        self.file = Some(Mutex::new(BufWriter::new(file)));
        Ok(self)
    }

    fn log(&self, level: LogLevel, message: &str) {
        if level < self.level {
            return;
        }

        let entry = LogEntry::new(level, &self.target, message.to_string());
        
        // Console output
        let color = match level {
            LogLevel::Trace => "",
            LogLevel::Debug => "\x1b[36m",
            LogLevel::Info => "\x1b[32m",
            LogLevel::Warn => "\x1b[33m",
            LogLevel::Error => "\x1b[31m",
        };
        let reset = "\x1b[0m";

        println!("{}{} [{}] [{}] {}: {}{}",
            color,
            entry.timestamp,
            level,
            self.target,
            entry.message,
            reset
        );

        // File output
        if let Some(ref file) = self.file {
            if let Ok(mut guard) = file.lock() {
                let _ = writeln!(guard, "[{}] [{}] {}: {}",
                    entry.timestamp,
                    level,
                    self.target,
                    entry.message
                );
            }
        }
    }

    pub fn trace(&self, message: &str) { self.log(LogLevel::Trace, message); }
    pub fn debug(&self, message: &str) { self.log(LogLevel::Debug, message); }
    pub fn info(&self, message: &str) { self.log(LogLevel::Info, message); }
    pub fn warn(&self, message: &str) { self.log(LogLevel::Warn, message); }
    pub fn error(&self, message: &str) { self.log(LogLevel::Error, message); }

    pub fn log_format(&self, level: LogLevel, format: &str, args: std::fmt::Arguments<'_>) {
        let message = format(format, args);
        self.log(level, &message);
    }

    pub fn trace_fmt(&self, format: &str, args: std::fmt::Arguments<'_>) { self.log_format(LogLevel::Trace, format, args); }
    pub fn debug_fmt(&self, format: &str, args: std::fmt::Arguments<'_>) { self.log_format(LogLevel::Debug, format, args); }
    pub fn info_fmt(&self, format: &str, args: std::fmt::Arguments<'_>) { self.log_format(LogLevel::Info, format, args); }
    pub fn warn_fmt(&self, format: &str, args: std::fmt::Arguments<'_>) { self.log_format(LogLevel::Warn, format, args); }
    pub fn error_fmt(&self, format: &str, args: std::fmt::Arguments<'_>) { self.log_format(LogLevel::Error, format, args); }
}

/// Global logger
lazy_static::lazy_static! {
    pub static ref GLOBAL_LOGGER: Logger = Logger::new("global");
}

/// Get global logger
pub fn global_logger() -> &'static Logger {
    &GLOBAL_LOGGER
}

/// Initialize global logger with file
pub fn init_logging(log_dir: PathBuf, level: LogLevel) -> std::io::Result<()> {
    std::fs::create_dir_all(&log_dir)?;
    let log_file = log_dir.join(format!("voxelnaut-{}.log", Local::now().format("%Y-%m-%d")));
    
    GLOBAL_LOGGER.with_level(level);
    
    if let Some(ref file) = GLOBAL_LOGGER.file {
        if let Ok(mut guard) = file.lock() {
            guard.flush().ok();
        }
    }
    
    Ok(())
}

/// Helper macros
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logging::GLOBAL_LOGGER.info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logging::GLOBAL_LOGGER.warn(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logging::GLOBAL_LOGGER.error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logging::GLOBAL_LOGGER.debug(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::logging::GLOBAL_LOGGER.trace(&format!($($arg)*))
    };
}