//! Kernel profiler
//! 
//! Profiling tool for kernel performance analysis

use std::collections::HashMap;

/// Kernel profiler
pub struct KernelProfiler {
    samples: HashMap<String, Vec<u64>>,
}

impl KernelProfiler {
    /// Create new profiler
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }
    
    /// Record sample
    pub fn record(&mut self, function: &str, duration: u64) {
        self.samples
            .entry(function.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    /// Get statistics
    pub fn statistics(&self) -> HashMap<String, ProfileStats> {
        self.samples
            .iter()
            .map(|(func, samples)| {
                let stats = ProfileStats {
                    count: samples.len(),
                    total: samples.iter().sum(),
                    average: samples.iter().sum::<u64>() / samples.len() as u64,
                    min: *samples.iter().min().unwrap(),
                    max: *samples.iter().max().unwrap(),
                };
                (func.clone(), stats)
            })
            .collect()
    }
}

/// Profile statistics
pub struct ProfileStats {
    pub count: usize,
    pub total: u64,
    pub average: u64,
    pub min: u64,
    pub max: u64,
}

