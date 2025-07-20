use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, trace, warn};
use sysinfo::System;

/// Debug levels for controlling verbosity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<DebugLevel> for tracing::Level {
    fn from(level: DebugLevel) -> Self {
        match level {
            DebugLevel::Trace => tracing::Level::TRACE,
            DebugLevel::Debug => tracing::Level::DEBUG,
            DebugLevel::Info => tracing::Level::INFO,
            DebugLevel::Warn => tracing::Level::WARN,
            DebugLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// Performance metrics for monitoring system performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub timestamp: f64,
}

/// Debug configuration that can be controlled via environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enabled: bool,
    pub level: DebugLevel,
    pub log_to_file: bool,
    pub log_file_path: String,
    pub performance_monitoring: bool,
    pub gps_debug: bool,
    pub ui_debug: bool,
    pub network_debug: bool,
    pub detailed_logging: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("YACHTPIT_DEBUG").unwrap_or_default() == "true",
            level: match std::env::var("YACHTPIT_DEBUG_LEVEL").unwrap_or_default().as_str() {
                "trace" => DebugLevel::Trace,
                "debug" => DebugLevel::Debug,
                "warn" => DebugLevel::Warn,
                "error" => DebugLevel::Error,
                _ => DebugLevel::Info,
            },
            log_to_file: std::env::var("YACHTPIT_LOG_FILE").unwrap_or_default() == "true",
            log_file_path: std::env::var("YACHTPIT_LOG_PATH").unwrap_or_else(|_| "yachtpit_debug.log".to_string()),
            performance_monitoring: std::env::var("YACHTPIT_PERF_MONITOR").unwrap_or_default() == "true",
            gps_debug: std::env::var("YACHTPIT_GPS_DEBUG").unwrap_or_default() == "true",
            ui_debug: std::env::var("YACHTPIT_UI_DEBUG").unwrap_or_default() == "true",
            network_debug: std::env::var("YACHTPIT_NETWORK_DEBUG").unwrap_or_default() == "true",
            detailed_logging: std::env::var("YACHTPIT_DETAILED_LOG").unwrap_or_default() == "true",
        }
    }
}

/// Debug service resource for managing debugging capabilities
#[derive(Resource)]
pub struct DebugService {
    pub config: DebugConfig,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub debug_data: HashMap<String, serde_json::Value>,
    pub system_info: System,
    pub start_time: Instant,
    pub last_perf_update: Instant,
}

impl Default for DebugService {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugService {
    pub fn new() -> Self {
        let config = DebugConfig::default();
        
        // Initialize tracing subscriber if debug is enabled
        if config.enabled {
            Self::init_tracing(&config);
        }

        Self {
            config,
            performance_metrics: None,
            debug_data: HashMap::new(),
            system_info: System::new_all(),
            start_time: Instant::now(),
            last_perf_update: Instant::now(),
        }
    }

    /// Initialize tracing subscriber with appropriate configuration
    fn init_tracing(config: &DebugConfig) {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                let level = match config.level {
                    DebugLevel::Trace => "trace",
                    DebugLevel::Debug => "debug",
                    DebugLevel::Info => "info",
                    DebugLevel::Warn => "warn",
                    DebugLevel::Error => "error",
                };
                EnvFilter::new(format!("yachtpit={}", level))
            });

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(config.detailed_logging)
                .with_line_number(config.detailed_logging));

        if config.log_to_file {
            let file_appender = tracing_appender::rolling::daily("logs", &config.log_file_path);
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            
            subscriber
                .with(tracing_subscriber::fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false))
                .init();
        } else {
            subscriber.init();
        }

        info!("Debug service initialized with level: {:?}", config.level);
    }

    /// Log debug information with context
    pub fn debug_log(&self, component: &str, message: &str, data: Option<serde_json::Value>) {
        if !self.config.enabled {
            return;
        }

        match data {
            Some(data) => debug!(component = component, data = ?data, "{}", message),
            None => debug!(component = component, "{}", message),
        }
    }

    /// Log GPS-specific debug information
    pub fn debug_gps(&self, message: &str, lat: f64, lon: f64, heading: Option<f64>, speed: Option<f64>) {
        if !self.config.enabled || !self.config.gps_debug {
            return;
        }

        debug!(
            component = "GPS",
            latitude = lat,
            longitude = lon,
            heading = heading,
            speed = speed,
            "{}",
            message
        );
    }

    /// Log performance metrics
    pub fn debug_performance(&mut self, fps: f32, frame_time_ms: f32) {
        if !self.config.enabled || !self.config.performance_monitoring {
            return;
        }

        // Update performance metrics every 5 seconds
        if self.last_perf_update.elapsed() >= Duration::from_secs(5) {
            self.system_info.refresh_all();
            
            let cpu_usage = self.system_info.global_cpu_info().cpu_usage();
            let memory_usage = self.system_info.used_memory();
            let memory_total = self.system_info.total_memory();

            let metrics = PerformanceMetrics {
                cpu_usage,
                memory_usage,
                memory_total,
                fps,
                frame_time_ms,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            };

            debug!(
                component = "PERFORMANCE",
                cpu_usage = cpu_usage,
                memory_usage = memory_usage,
                memory_total = memory_total,
                fps = fps,
                frame_time_ms = frame_time_ms,
                "Performance metrics updated"
            );

            self.performance_metrics = Some(metrics);
            self.last_perf_update = Instant::now();
        }
    }

    /// Store arbitrary debug data
    pub fn store_debug_data(&mut self, key: String, value: serde_json::Value) {
        if self.config.enabled {
            self.debug_data.insert(key, value);
        }
    }

    /// Get stored debug data
    pub fn get_debug_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.debug_data.get(key)
    }

    /// Get all debug data as JSON
    pub fn get_all_debug_data(&self) -> serde_json::Value {
        serde_json::json!({
            "config": self.config,
            "performance_metrics": self.performance_metrics,
            "debug_data": self.debug_data,
            "uptime_seconds": self.start_time.elapsed().as_secs(),
        })
    }

    /// Log system information
    pub fn log_system_info(&mut self) {
        if !self.config.enabled {
            return;
        }

        self.system_info.refresh_all();
        
        info!(
            component = "SYSTEM",
            os = System::name().unwrap_or_default(),
            kernel_version = System::kernel_version().unwrap_or_default(),
            cpu_count = self.system_info.cpus().len(),
            total_memory = self.system_info.total_memory(),
            "System information"
        );
    }

    /// Create a debug snapshot of current state
    pub fn create_debug_snapshot(&mut self) -> serde_json::Value {
        if !self.config.enabled {
            return serde_json::json!({});
        }

        self.system_info.refresh_all();
        
        serde_json::json!({
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            "uptime_seconds": self.start_time.elapsed().as_secs(),
            "config": self.config,
            "performance": self.performance_metrics,
            "system": {
                "cpu_usage": self.system_info.global_cpu_info().cpu_usage(),
                "memory_usage": self.system_info.used_memory(),
                "memory_total": self.system_info.total_memory(),
                "process_count": self.system_info.processes().len(),
            },
            "debug_data": self.debug_data,
        })
    }

    /// Enable/disable debug mode at runtime
    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        if enabled {
            info!("Debug mode enabled at runtime");
        } else {
            info!("Debug mode disabled at runtime");
        }
    }

    /// Change debug level at runtime
    pub fn set_debug_level(&mut self, level: DebugLevel) {
        self.config.level = level;
        info!("Debug level changed to: {:?}", level);
    }
}

/// System for updating debug service
pub fn update_debug_service(
    mut debug_service: ResMut<DebugService>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
) {
    if !debug_service.config.enabled {
        return;
    }

    // Get FPS and frame time from Bevy diagnostics
    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0) as f32;

    let frame_time = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|frame_time| frame_time.smoothed())
        .unwrap_or(0.0) as f32 * 1000.0; // Convert to milliseconds

    debug_service.debug_performance(fps, frame_time);
}

/// Debug service plugin
pub struct DebugServicePlugin;

impl Plugin for DebugServicePlugin {
    fn build(&self, app: &mut App) {
        let debug_service = DebugService::new();
        
        // Log system info on startup if debug is enabled
        if debug_service.config.enabled {
            info!("YachtPit Debug Service starting up...");
        }

        app.insert_resource(debug_service)
            .add_systems(Update, update_debug_service);
    }
}

/// Macro for easy debug logging
#[macro_export]
macro_rules! debug_log {
    ($debug_service:expr, $component:expr, $message:expr) => {
        $debug_service.debug_log($component, $message, None);
    };
    ($debug_service:expr, $component:expr, $message:expr, $data:expr) => {
        $debug_service.debug_log($component, $message, Some($data));
    };
}

/// Macro for GPS debug logging
#[macro_export]
macro_rules! debug_gps {
    ($debug_service:expr, $message:expr, $lat:expr, $lon:expr) => {
        $debug_service.debug_gps($message, $lat, $lon, None, None);
    };
    ($debug_service:expr, $message:expr, $lat:expr, $lon:expr, $heading:expr, $speed:expr) => {
        $debug_service.debug_gps($message, $lat, $lon, $heading, $speed);
    };
}