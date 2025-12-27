//! Resource limits and monitoring for sandboxes.

use serde::{Deserialize, Serialize};

/// Resource limits for a sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes.
    pub memory_bytes: u64,

    /// Maximum CPU time in milliseconds.
    pub cpu_time_ms: u64,

    /// Maximum wall clock time in seconds.
    pub wall_time_seconds: u32,

    /// Maximum disk space in bytes.
    pub disk_bytes: u64,

    /// Maximum number of processes.
    pub max_processes: u32,

    /// Maximum number of open files.
    pub max_open_files: u32,

    /// Maximum network bandwidth (bytes per second).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_bandwidth_bps: Option<u64>,

    /// Maximum number of network connections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 512 * 1024 * 1024, // 512 MB
            cpu_time_ms: 60_000,             // 60 seconds CPU time
            wall_time_seconds: 300,          // 5 minutes wall time
            disk_bytes: 1024 * 1024 * 1024,  // 1 GB
            max_processes: 32,
            max_open_files: 256,
            network_bandwidth_bps: None,
            max_connections: Some(100),
        }
    }
}

impl ResourceLimits {
    /// Create minimal limits for quick tasks.
    pub fn minimal() -> Self {
        Self {
            memory_bytes: 128 * 1024 * 1024, // 128 MB
            cpu_time_ms: 5_000,              // 5 seconds
            wall_time_seconds: 30,           // 30 seconds
            disk_bytes: 100 * 1024 * 1024,   // 100 MB
            max_processes: 4,
            max_open_files: 32,
            network_bandwidth_bps: None,
            max_connections: Some(10),
        }
    }

    /// Create generous limits for long-running tasks.
    pub fn generous() -> Self {
        Self {
            memory_bytes: 4 * 1024 * 1024 * 1024, // 4 GB
            cpu_time_ms: 600_000,                 // 10 minutes CPU time
            wall_time_seconds: 3600,              // 1 hour wall time
            disk_bytes: 10 * 1024 * 1024 * 1024,  // 10 GB
            max_processes: 128,
            max_open_files: 1024,
            network_bandwidth_bps: None,
            max_connections: Some(500),
        }
    }

    /// Set memory limit.
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = bytes;
        self
    }

    /// Set CPU time limit.
    pub fn with_cpu_time(mut self, ms: u64) -> Self {
        self.cpu_time_ms = ms;
        self
    }

    /// Set wall time limit.
    pub fn with_wall_time(mut self, seconds: u32) -> Self {
        self.wall_time_seconds = seconds;
        self
    }
}

/// Current resource usage of a sandbox.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Current memory usage in bytes.
    pub memory_bytes: u64,

    /// Peak memory usage in bytes.
    pub peak_memory_bytes: u64,

    /// Total CPU time used in milliseconds.
    pub cpu_time_ms: u64,

    /// Elapsed wall clock time in milliseconds.
    pub wall_time_ms: u64,

    /// Current disk usage in bytes.
    pub disk_bytes: u64,

    /// Current number of processes.
    pub process_count: u32,

    /// Current number of open files.
    pub open_file_count: u32,

    /// Total bytes sent over network.
    pub network_bytes_sent: u64,

    /// Total bytes received over network.
    pub network_bytes_received: u64,

    /// Current number of network connections.
    pub connection_count: u32,
}

impl ResourceUsage {
    /// Check if usage exceeds limits.
    pub fn exceeds(&self, limits: &ResourceLimits) -> Option<ResourceViolation> {
        if self.memory_bytes > limits.memory_bytes {
            return Some(ResourceViolation::MemoryExceeded {
                used: self.memory_bytes,
                limit: limits.memory_bytes,
            });
        }

        if self.cpu_time_ms > limits.cpu_time_ms {
            return Some(ResourceViolation::CpuTimeExceeded {
                used_ms: self.cpu_time_ms,
                limit_ms: limits.cpu_time_ms,
            });
        }

        let wall_time_limit_ms = limits.wall_time_seconds as u64 * 1000;
        if self.wall_time_ms > wall_time_limit_ms {
            return Some(ResourceViolation::WallTimeExceeded {
                used_ms: self.wall_time_ms,
                limit_ms: wall_time_limit_ms,
            });
        }

        if self.disk_bytes > limits.disk_bytes {
            return Some(ResourceViolation::DiskExceeded {
                used: self.disk_bytes,
                limit: limits.disk_bytes,
            });
        }

        if self.process_count > limits.max_processes {
            return Some(ResourceViolation::ProcessLimitExceeded {
                count: self.process_count,
                limit: limits.max_processes,
            });
        }

        if self.open_file_count > limits.max_open_files {
            return Some(ResourceViolation::OpenFileLimitExceeded {
                count: self.open_file_count,
                limit: limits.max_open_files,
            });
        }

        if let Some(max_conn) = limits.max_connections {
            if self.connection_count > max_conn {
                return Some(ResourceViolation::ConnectionLimitExceeded {
                    count: self.connection_count,
                    limit: max_conn,
                });
            }
        }

        None
    }

    /// Get usage as percentages of limits.
    pub fn as_percentages(&self, limits: &ResourceLimits) -> ResourcePercentages {
        ResourcePercentages {
            memory: (self.memory_bytes as f64 / limits.memory_bytes as f64 * 100.0) as u8,
            cpu_time: (self.cpu_time_ms as f64 / limits.cpu_time_ms as f64 * 100.0) as u8,
            wall_time: (self.wall_time_ms as f64 / (limits.wall_time_seconds as f64 * 1000.0)
                * 100.0) as u8,
            disk: (self.disk_bytes as f64 / limits.disk_bytes as f64 * 100.0) as u8,
        }
    }
}

/// Resource usage as percentages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePercentages {
    /// Memory usage percentage.
    pub memory: u8,
    /// CPU time usage percentage.
    pub cpu_time: u8,
    /// Wall time usage percentage.
    pub wall_time: u8,
    /// Disk usage percentage.
    pub disk: u8,
}

/// A resource limit violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResourceViolation {
    /// Memory limit exceeded.
    MemoryExceeded { used: u64, limit: u64 },
    /// CPU time limit exceeded.
    CpuTimeExceeded { used_ms: u64, limit_ms: u64 },
    /// Wall clock time limit exceeded.
    WallTimeExceeded { used_ms: u64, limit_ms: u64 },
    /// Disk space limit exceeded.
    DiskExceeded { used: u64, limit: u64 },
    /// Process limit exceeded.
    ProcessLimitExceeded { count: u32, limit: u32 },
    /// Open file limit exceeded.
    OpenFileLimitExceeded { count: u32, limit: u32 },
    /// Connection limit exceeded.
    ConnectionLimitExceeded { count: u32, limit: u32 },
    /// Network bandwidth limit exceeded.
    BandwidthExceeded { bps: u64, limit_bps: u64 },
}

impl std::fmt::Display for ResourceViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryExceeded { used, limit } => {
                write!(
                    f,
                    "Memory limit exceeded: {} bytes used, {} bytes limit",
                    used, limit
                )
            }
            Self::CpuTimeExceeded { used_ms, limit_ms } => {
                write!(
                    f,
                    "CPU time limit exceeded: {}ms used, {}ms limit",
                    used_ms, limit_ms
                )
            }
            Self::WallTimeExceeded { used_ms, limit_ms } => {
                write!(
                    f,
                    "Wall time limit exceeded: {}ms used, {}ms limit",
                    used_ms, limit_ms
                )
            }
            Self::DiskExceeded { used, limit } => {
                write!(
                    f,
                    "Disk limit exceeded: {} bytes used, {} bytes limit",
                    used, limit
                )
            }
            Self::ProcessLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Process limit exceeded: {} processes, {} limit",
                    count, limit
                )
            }
            Self::OpenFileLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Open file limit exceeded: {} files, {} limit",
                    count, limit
                )
            }
            Self::ConnectionLimitExceeded { count, limit } => {
                write!(
                    f,
                    "Connection limit exceeded: {} connections, {} limit",
                    count, limit
                )
            }
            Self::BandwidthExceeded { bps, limit_bps } => {
                write!(
                    f,
                    "Bandwidth limit exceeded: {} bps, {} bps limit",
                    bps, limit_bps
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.memory_bytes, 512 * 1024 * 1024);
        assert_eq!(limits.wall_time_seconds, 300);
    }

    #[test]
    fn test_limits_builder() {
        let limits = ResourceLimits::minimal()
            .with_memory(256 * 1024 * 1024)
            .with_wall_time(60);

        assert_eq!(limits.memory_bytes, 256 * 1024 * 1024);
        assert_eq!(limits.wall_time_seconds, 60);
    }

    #[test]
    fn test_usage_within_limits() {
        let limits = ResourceLimits::default();
        let usage = ResourceUsage {
            memory_bytes: 100 * 1024 * 1024,
            cpu_time_ms: 1000,
            wall_time_ms: 5000,
            ..Default::default()
        };

        assert!(usage.exceeds(&limits).is_none());
    }

    #[test]
    fn test_usage_exceeds_memory() {
        let limits = ResourceLimits::minimal();
        let usage = ResourceUsage {
            memory_bytes: 200 * 1024 * 1024, // 200 MB > 128 MB limit
            ..Default::default()
        };

        let violation = usage.exceeds(&limits);
        assert!(matches!(
            violation,
            Some(ResourceViolation::MemoryExceeded { .. })
        ));
    }

    #[test]
    fn test_usage_percentages() {
        let limits = ResourceLimits {
            memory_bytes: 100,
            cpu_time_ms: 100,
            wall_time_seconds: 10, // 10000 ms
            disk_bytes: 100,
            ..Default::default()
        };

        let usage = ResourceUsage {
            memory_bytes: 50,
            cpu_time_ms: 25,
            wall_time_ms: 5000,
            disk_bytes: 75,
            ..Default::default()
        };

        let pct = usage.as_percentages(&limits);
        assert_eq!(pct.memory, 50);
        assert_eq!(pct.cpu_time, 25);
        assert_eq!(pct.wall_time, 50);
        assert_eq!(pct.disk, 75);
    }
}
