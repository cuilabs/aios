//! Scheduler benchmarks

#![no_std]
#![feature(test)]

extern crate test;

use aios_kernel_agent::scheduler::AgentScheduler;

#[bench]
fn bench_schedule_agent(b: &mut test::Bencher) {
    let scheduler = AgentScheduler::new();
    scheduler.add_agent(1, 100, 10);
    
    b.iter(|| {
        let next = scheduler.next();
        test::black_box(next);
    });
}

