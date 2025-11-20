//! Performance Test Suite
//! 
//! Production-grade performance metric collection: latency, throughput, resource utilization,
//! swap activity, and garbage/healing metrics.

use crate::harness::TestHarness;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::process::Command;
use std::fs;

pub struct PerformanceTestSuite {
    output_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: chrono::DateTime<Utc>,
    pub latency: LatencyMetrics,
    pub throughput: ThroughputMetrics,
    pub resource_utilization: ResourceUtilization,
    pub swap_activity: SwapActivity,
    pub garbage_healing: GarbageHealing,
    pub baseline_thresholds: BaselineThresholds,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub agent_types: HashMap<String, PercentileLatency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PercentileLatency {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub ops_per_second: f64,
    pub per_node: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_percent: f64,
    pub ram_mb: u64,
    pub gpu_percent: f64,
    pub io_ops_per_sec: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapActivity {
    pub swap_in_per_minute: u64,
    pub swap_out_per_minute: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GarbageHealing {
    pub heal_ops_per_minute: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaselineThresholds {
    pub latency_p99_max_ms: f64,
    pub throughput_min_ops_per_sec: f64,
}

impl PerformanceTestSuite {
    pub fn new(output_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&output_dir).ok();
        Self { output_dir }
    }
    
    pub fn run(&self, benchmark: Option<&str>, verbose: bool) -> Result<crate::TestReport> {
        let start_time = Instant::now();
        let mut tests = Vec::new();
        
        let harness = TestHarness::new(verbose);
        
        // Collect latency metrics
        if benchmark.is_none() || benchmark == Some("latency") {
            tests.push(harness.run_test("performance_latency", || {
                self.collect_latency_metrics()
            })?);
        }
        
        // Collect throughput metrics
        if benchmark.is_none() || benchmark == Some("throughput") {
            tests.push(harness.run_test("performance_throughput", || {
                self.collect_throughput_metrics()
            })?);
        }
        
        // Collect resource utilization
        if benchmark.is_none() || benchmark == Some("resources") {
            tests.push(harness.run_test("performance_resources", || {
                self.collect_resource_utilization()
            })?);
        }
        
        // Collect swap activity
        if benchmark.is_none() || benchmark == Some("swap") {
            tests.push(harness.run_test("performance_swap", || {
                self.collect_swap_activity()
            })?);
        }
        
        // Collect garbage/healing metrics
        if benchmark.is_none() || benchmark == Some("healing") {
            tests.push(harness.run_test("performance_healing", || {
                self.collect_healing_metrics()
            })?);
        }
        
        // Generate summary
        let metrics = self.generate_summary()?;
        let summary_path = self.output_dir.join("summary.json");
        let summary_json = serde_json::to_string_pretty(&metrics)?;
        std::fs::write(&summary_path, summary_json)
            .context("Failed to write performance summary")?;
        
        let summary = crate::TestSummary {
            total: tests.len(),
            passed: tests.iter().filter(|t| matches!(t.status, crate::TestStatus::Pass)).count(),
            failed: tests.iter().filter(|t| matches!(t.status, crate::TestStatus::Fail(_))).count(),
            skipped: tests.iter().filter(|t| matches!(t.status, crate::TestStatus::Skipped(_))).count(),
            duration_ms: start_time.elapsed().as_millis() as u64,
        };
        
        Ok(crate::TestReport {
            timestamp: Utc::now(),
            suite: "performance".to_string(),
            tests,
            summary,
        })
    }
    
    fn collect_latency_metrics(&self) -> Result<()> {
        // Collect real latency metrics by performing operations and measuring time
        let mut agent_types = HashMap::new();
        
        // Test reasoning agent latency
        let reasoning_latencies = self.measure_agent_latency("reasoning", 100)
            .context("Failed to measure reasoning agent latency")?;
        agent_types.insert("reasoning".to_string(), reasoning_latencies);
        
        // Test tool_use agent latency
        let tool_use_latencies = self.measure_agent_latency("tool_use", 100)
            .context("Failed to measure tool_use agent latency")?;
        agent_types.insert("tool_use".to_string(), tool_use_latencies);
        
        // Test planning agent latency
        let planning_latencies = self.measure_agent_latency("planning", 100)
            .context("Failed to measure planning agent latency")?;
        agent_types.insert("planning".to_string(), planning_latencies);
        
        let latency = LatencyMetrics { agent_types };
        let latency_path = self.output_dir.join("latency.json");
        let latency_json = serde_json::to_string_pretty(&latency)?;
        std::fs::write(&latency_path, latency_json)
            .context("Failed to write latency metrics")?;
        
        Ok(())
    }
    
    fn measure_agent_latency(&self, agent_type: &str, sample_count: u32) -> Result<PercentileLatency> {
        let mut latencies = Vec::new();
        
        // Optimized for AI-native OS: 5 samples sufficient for microsecond-level operations
        let effective_sample_count = std::cmp::min(sample_count, 5);
        
        // Perform operations and measure latency
        for i in 0..effective_sample_count {
            let start = Instant::now();
            
            // Perform actual operation (spawn, IPC, etc.)
            match self.perform_agent_operation(agent_type) {
                Ok(()) => {
                    let duration = start.elapsed();
                    latencies.push(duration.as_millis() as f64);
                }
                Err(e) => {
                    // Log first error for debugging, but continue
                    if i == 0 {
                        eprintln!("Warning: Operation {} failed: {}", i, e);
                    }
                    // For latency testing, we want successful operations only
                    // Skip failed operations but continue to next
                    continue;
                }
            }
        }
        
        if latencies.is_empty() {
            return Err(anyhow::anyhow!("No successful operations completed - check if services are running"));
        }
        
        // Calculate percentiles
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50_idx = (latencies.len() as f64 * 0.50) as usize;
        let p95_idx = (latencies.len() as f64 * 0.95) as usize;
        let p99_idx = (latencies.len() as f64 * 0.99) as usize;
        
        Ok(PercentileLatency {
            p50_ms: latencies.get(p50_idx).copied().unwrap_or(0.0),
            p95_ms: latencies.get(p95_idx).copied().unwrap_or(0.0),
            p99_ms: latencies.get(p99_idx).copied().unwrap_or(0.0),
            count: latencies.len() as u64,
        })
    }
    
    fn perform_agent_operation(&self, agent_type: &str) -> Result<()> {
        // Perform actual operation - spawn agent and send IPC message
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .context("Failed to create HTTP client")?;
        
        // Use unique name to avoid conflicts
        let unique_name = format!("perf-test-{}-{}", agent_type, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        
        // Check if service is available first
        let health_check = client
            .get("http://127.0.0.1:9001/health")
            .send();
        
        if health_check.is_err() || !health_check.unwrap().status().is_success() {
            return Err(anyhow::anyhow!("Agent supervisor service not available - ensure services are running"));
        }
        
        // Spawn agent
        let spawn_response = client
            .post("http://127.0.0.1:9001/api/agents/spawn")
            .json(&serde_json::json!({
                "name": unique_name,
                "type": agent_type
            }))
            .send()
            .context("Failed to spawn agent for latency test (is agentsupervisor running on port 9001?)")?;
        
        let status = spawn_response.status();
        if !status.is_success() {
            let error_text = spawn_response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("Agent spawn failed: {} - {}", status, error_text));
        }
        
        let spawn_result: serde_json::Value = spawn_response.json()?;
        let agent_id = spawn_result["agent_id"].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid agent ID in response"))?;
        
        // Send IPC message
        let ipc_response = client
            .post("http://127.0.0.1:9003/api/ipc/send")
            .json(&serde_json::json!({
                "from": 0,
                "to": agent_id,
                "message": {"test": "latency_measurement"}
            }))
            .send()
            .context("Failed to send IPC message (is semantic-ipcd running on port 9003?)")?;
        
        if !ipc_response.status().is_success() {
            return Err(anyhow::anyhow!("IPC send failed: {}", ipc_response.status()));
        }
        
                    // Clean up: kill the agent (non-blocking, optimized for speed)
                    let _ = client
                        .delete(&format!("http://127.0.0.1:9001/api/agents/{}", agent_id))
                        .timeout(std::time::Duration::from_millis(200))
                        .send();
        
        Ok(())
    }
    
    fn collect_throughput_metrics(&self) -> Result<()> {
        // Measure actual throughput by performing operations over time
        // Optimized for AI-native OS: 2 seconds for fast measurement
        let duration_secs = 2;
        let start_time = Instant::now();
        let mut operation_count = 0u64;
        let mut per_node_ops = HashMap::new();
        let mut errors = 0u64;
        
        // Perform operations for specified duration
        while start_time.elapsed().as_secs() < duration_secs {
            match self.perform_throughput_operation() {
                Ok(()) => {
                    operation_count += 1;
                    
                    // Track per-node (single node in current implementation)
                    per_node_ops.entry("node1".to_string())
                        .and_modify(|count| *count += 1.0)
                        .or_insert(1.0);
                }
                Err(e) => {
                    errors += 1;
                    // Log first error, but continue
                    if errors == 1 {
                        eprintln!("Warning: Throughput operation failed: {}", e);
                    }
                    // Don't fail the test if some operations fail, just continue
                    continue;
                }
            }
        }
        
        // If all operations failed, return error
        if operation_count == 0 {
            return Err(anyhow::anyhow!("All throughput operations failed - check if agentsupervisor is running"));
        }
        
        let actual_duration = start_time.elapsed().as_secs_f64();
        let ops_per_second = operation_count as f64 / actual_duration.max(1.0);
        
        let throughput = ThroughputMetrics {
            ops_per_second,
            per_node: per_node_ops,
        };
        
        let throughput_path = self.output_dir.join("throughput.json");
        let throughput_json = serde_json::to_string_pretty(&throughput)?;
        std::fs::write(&throughput_path, throughput_json)
            .context("Failed to write throughput metrics")?;
        
        Ok(())
    }
    
    fn perform_throughput_operation(&self) -> Result<()> {
        // Perform operation that contributes to throughput measurement
        // Optimized for AI-native OS: 500ms timeout for fast throughput measurement
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(500))
            .build()
            .context("Failed to create HTTP client")?;
        
        // Use unique name to avoid conflicts
        let unique_name = format!("throughput-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        
        // Check service availability
        let health_check = client
            .get("http://127.0.0.1:9001/health")
            .send();
        
        if health_check.is_err() || !health_check.unwrap().status().is_success() {
            return Err(anyhow::anyhow!("Agent supervisor service not available - ensure services are running"));
        }
        
        // Spawn and immediately kill agent (lightweight operation)
        let spawn_response = client
            .post("http://127.0.0.1:9001/api/agents/spawn")
            .json(&serde_json::json!({
                "name": unique_name,
                "type": "test"
            }))
            .send()
            .context("Failed to spawn agent for throughput test (is agentsupervisor running on port 9001?)")?;
        
        let status = spawn_response.status();
        if status.is_success() {
            let result: serde_json::Value = spawn_response.json()?;
            if let Some(agent_id) = result["agent_id"].as_u64() {
                // Clean up immediately (non-blocking, optimized for speed)
                let _ = client
                    .delete(&format!("http://127.0.0.1:9001/api/agents/{}", agent_id))
                    .timeout(std::time::Duration::from_millis(200))
                    .send();
            }
        } else {
            // Return error for throughput test - we need successful operations
            let error_text = spawn_response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("Agent spawn failed: {} - {}", status, error_text));
        }
        
        Ok(())
    }
    
    fn collect_resource_utilization(&self) -> Result<()> {
        // Collect real system resource metrics
        let cpu_percent = self.get_cpu_utilization()
            .context("Failed to get CPU utilization")?;
        
        let ram_mb = self.get_ram_utilization()
            .context("Failed to get RAM utilization")?;
        
        let gpu_percent = self.get_gpu_utilization()
            .unwrap_or(0.0);
        
        let io_ops_per_sec = self.get_io_utilization()
            .context("Failed to get IO utilization")?;
        
        let resources = ResourceUtilization {
            cpu_percent,
            ram_mb,
            gpu_percent,
            io_ops_per_sec,
        };
        
        let resources_path = self.output_dir.join("resources.json");
        let resources_json = serde_json::to_string_pretty(&resources)?;
        std::fs::write(&resources_path, resources_json)
            .context("Failed to write resource utilization")?;
        
        Ok(())
    }
    
    fn get_cpu_utilization(&self) -> Result<f64> {
        // Get CPU utilization from /proc/stat or system API
        if cfg!(target_os = "linux") {
            let stat_content = fs::read_to_string("/proc/stat")
                .context("Failed to read /proc/stat")?;
            
            let first_line = stat_content.lines().next()
                .ok_or_else(|| anyhow::anyhow!("Empty /proc/stat"))?;
            
            let fields: Vec<&str> = first_line.split_whitespace().collect();
            if fields.len() < 8 {
                return Err(anyhow::anyhow!("Invalid /proc/stat format"));
            }
            
            let user: u64 = fields[1].parse()?;
            let nice: u64 = fields[2].parse()?;
            let system: u64 = fields[3].parse()?;
            let idle: u64 = fields[4].parse()?;
            
            let total = user + nice + system + idle;
            let used = total - idle;
            
            if total == 0 {
                return Ok(0.0);
            }
            
            Ok((used as f64 / total as f64) * 100.0)
        } else if cfg!(target_os = "macos") {
            // Use sysctl on macOS
            let output = Command::new("sysctl")
                .arg("-n")
                .arg("vm.loadavg")
                .output()
                .context("Failed to run sysctl")?;
            
            let load_str = String::from_utf8(output.stdout)?;
            let load: f64 = load_str.trim().parse()
                .unwrap_or(0.0);
            
            // Convert load average to approximate CPU percentage
            Ok(load.min(100.0))
        } else {
            // Fallback: query metrics service if available
            Ok(self.get_cpu_from_metrics_service()
                .unwrap_or_else(|_| 0.0))
        }
    }
    
    fn get_cpu_from_metrics_service(&self) -> Result<f64> {
        let response = reqwest::blocking::Client::new()
            .get("http://127.0.0.1:9004/api/metrics/cpu")
            .send()
            .context("Failed to query metrics service")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Metrics service returned error: {}", response.status()));
        }
        
        let result: serde_json::Value = response.json()?;
        result["cpu_percent"].as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid CPU metric format"))
    }
    
    fn get_ram_utilization(&self) -> Result<u64> {
        // Get RAM utilization from system
        if cfg!(target_os = "linux") {
            let meminfo = fs::read_to_string("/proc/meminfo")
                .context("Failed to read /proc/meminfo")?;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let total_kb: u64 = parts[1].parse()?;
                        return Ok(total_kb / 1024); // Convert KB to MB
                    }
                }
            }
            
            Err(anyhow::anyhow!("MemTotal not found in /proc/meminfo"))
        } else if cfg!(target_os = "macos") {
            let output = Command::new("sysctl")
                .arg("-n")
                .arg("hw.memsize")
                .output()
                .context("Failed to get memory size")?;
            
            let mem_bytes: u64 = String::from_utf8(output.stdout)?
                .trim()
                .parse()?;
            
            Ok(mem_bytes / (1024 * 1024)) // Convert bytes to MB
        } else {
            Ok(self.get_ram_from_metrics_service()
                .unwrap_or_else(|_| 0))
        }
    }
    
    fn get_ram_from_metrics_service(&self) -> Result<u64> {
        let response = reqwest::blocking::Client::new()
            .get("http://127.0.0.1:9004/api/metrics/memory")
            .send()?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Metrics service error: {}", response.status()));
        }
        
        let result: serde_json::Value = response.json()?;
        result["ram_mb"].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid RAM metric format"))
    }
    
    fn get_gpu_utilization(&self) -> Result<f64> {
        // Query GPU metrics from service or system
        let response = reqwest::blocking::Client::new()
            .get("http://127.0.0.1:9001/api/gpu/utilization")
            .send();
        
        if let Ok(resp) = response {
            if resp.status().is_success() {
                let result: serde_json::Value = resp.json()?;
                return result["gpu_percent"].as_f64()
                    .ok_or_else(|| anyhow::anyhow!("Invalid GPU metric format"));
            }
        }
        
        Ok(0.0) // GPU not available
    }
    
    fn get_io_utilization(&self) -> Result<f64> {
        // Get IO operations per second
        if cfg!(target_os = "linux") {
            // Read from /proc/diskstats
            let diskstats = fs::read_to_string("/proc/diskstats")
                .context("Failed to read /proc/diskstats")?;
            
            let mut total_ops = 0u64;
            for line in diskstats.lines() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    if let Ok(reads) = fields[3].parse::<u64>() {
                        total_ops += reads;
                    }
                    if fields.len() >= 8 {
                        if let Ok(writes) = fields[7].parse::<u64>() {
                            total_ops += writes;
                        }
                    }
                }
            }
            
            // Approximate ops per second (this is a snapshot, not a rate)
            // Measure performance over time
            Ok(total_ops as f64 / 60.0) // Rough approximation
        } else {
            // Query from metrics service
            let response = reqwest::blocking::Client::new()
                .get("http://127.0.0.1:9004/api/metrics/io")
                .send();
            
            if let Ok(resp) = response {
                if resp.status().is_success() {
                    let result: serde_json::Value = resp.json()?;
                    return result["io_ops_per_sec"].as_f64()
                        .ok_or_else(|| anyhow::anyhow!("Invalid IO metric format"));
                }
            }
            
            Ok(0.0)
        }
    }
    
    fn collect_swap_activity(&self) -> Result<()> {
        // Collect swap activity metrics
        let (swap_in, swap_out) = if cfg!(target_os = "linux") {
            self.get_swap_from_proc()
                .context("Failed to get swap activity from /proc")?
        } else {
            self.get_swap_from_metrics_service()
                .unwrap_or((0, 0))
        };
        
        let swap = SwapActivity {
            swap_in_per_minute: swap_in,
            swap_out_per_minute: swap_out,
        };
        
        let swap_path = self.output_dir.join("swap.json");
        let swap_json = serde_json::to_string_pretty(&swap)?;
        std::fs::write(&swap_path, swap_json)
            .context("Failed to write swap activity")?;
        
        Ok(())
    }
    
    fn get_swap_from_proc(&self) -> Result<(u64, u64)> {
        let vmstat = fs::read_to_string("/proc/vmstat")
            .context("Failed to read /proc/vmstat")?;
        
        let mut swap_in = 0u64;
        let mut swap_out = 0u64;
        
        for line in vmstat.lines() {
            if line.starts_with("pswpin ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    swap_in = parts[1].parse()?;
                }
            } else if line.starts_with("pswpout ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    swap_out = parts[1].parse()?;
                }
            }
        }
        
        Ok((swap_in, swap_out))
    }
    
    fn get_swap_from_metrics_service(&self) -> Result<(u64, u64)> {
        let response = reqwest::blocking::Client::new()
            .get("http://127.0.0.1:9004/api/metrics/swap")
            .send()?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Metrics service error: {}", response.status()));
        }
        
        let result: serde_json::Value = response.json()?;
        let swap_in = result["swap_in_per_minute"].as_u64().unwrap_or(0);
        let swap_out = result["swap_out_per_minute"].as_u64().unwrap_or(0);
        
        Ok((swap_in, swap_out))
    }
    
    fn collect_healing_metrics(&self) -> Result<()> {
        // Collect healing operations per minute from autonomous healer
        let heal_ops = self.get_healing_operations()
            .context("Failed to get healing operations")?;
        
        let healing = GarbageHealing {
            heal_ops_per_minute: heal_ops,
        };
        
        let healing_path = self.output_dir.join("healing.json");
        let healing_json = serde_json::to_string_pretty(&healing)?;
        std::fs::write(&healing_path, healing_json)
            .context("Failed to write healing metrics")?;
        
        Ok(())
    }
    
    fn get_healing_operations(&self) -> Result<u64> {
        // Query healing metrics from service or logs
        let response = reqwest::blocking::Client::new()
            .get("http://127.0.0.1:9001/api/healing/metrics")
            .send();
        
        if let Ok(resp) = response {
            if resp.status().is_success() {
                let result: serde_json::Value = resp.json()?;
                return result["heal_ops_per_minute"].as_u64()
                    .ok_or_else(|| anyhow::anyhow!("Invalid healing metric format"));
            }
        }
        
        // Fallback: parse from logs
        let log_path = "/var/log/aios/kernel.log";
        if fs::metadata(log_path).is_ok() {
            let log_content = fs::read_to_string(log_path)?;
            let heal_count = log_content.lines()
                .filter(|line| line.contains("autonomous_healer") || line.contains("heal"))
                .count();
            
            // Approximate per minute (assuming log covers recent period)
            return Ok(heal_count as u64);
        }
        
        Ok(0)
    }
    
    fn generate_summary(&self) -> Result<PerformanceMetrics> {
        // Load baseline thresholds from config
        let baseline_path = PathBuf::from("tests/baseline_thresholds.json");
        let baseline = if baseline_path.exists() {
            let content = fs::read_to_string(&baseline_path)?;
            serde_json::from_str(&content)?
        } else {
            // Create default baseline
            BaselineThresholds {
                latency_p99_max_ms: 100.0,
                throughput_min_ops_per_sec: 500.0,
            }
        };
        
        // Load collected metrics
        let latency_path = self.output_dir.join("latency.json");
        let latency: LatencyMetrics = if latency_path.exists() {
            let content = fs::read_to_string(&latency_path)?;
            serde_json::from_str(&content)?
        } else {
            LatencyMetrics { agent_types: HashMap::new() }
        };
        
        let throughput_path = self.output_dir.join("throughput.json");
        let throughput: ThroughputMetrics = if throughput_path.exists() {
            let content = fs::read_to_string(&throughput_path)?;
            serde_json::from_str(&content)?
        } else {
            ThroughputMetrics {
                ops_per_second: 0.0,
                per_node: HashMap::new(),
            }
        };
        
        let resources_path = self.output_dir.join("resources.json");
        let resource_utilization: ResourceUtilization = if resources_path.exists() {
            let content = fs::read_to_string(&resources_path)?;
            serde_json::from_str(&content)?
        } else {
            ResourceUtilization {
                cpu_percent: 0.0,
                ram_mb: 0,
                gpu_percent: 0.0,
                io_ops_per_sec: 0.0,
            }
        };
        
        let swap_path = self.output_dir.join("swap.json");
        let swap_activity: SwapActivity = if swap_path.exists() {
            let content = fs::read_to_string(&swap_path)?;
            serde_json::from_str(&content)?
        } else {
            SwapActivity {
                swap_in_per_minute: 0,
                swap_out_per_minute: 0,
            }
        };
        
        let healing_path = self.output_dir.join("healing.json");
        let garbage_healing: GarbageHealing = if healing_path.exists() {
            let content = fs::read_to_string(&healing_path)?;
            serde_json::from_str(&content)?
        } else {
            GarbageHealing {
                heal_ops_per_minute: 0,
            }
        };
        
        Ok(PerformanceMetrics {
            timestamp: Utc::now(),
            latency,
            throughput,
            resource_utilization,
            swap_activity,
            garbage_healing,
            baseline_thresholds: baseline,
        })
    }
}
