//! Memory subsystem benchmarks

#![no_std]
#![feature(test)]

extern crate test;

use aios_kernel_core::memory::{allocate_agent_memory, deallocate_agent_memory};

#[bench]
fn bench_allocate_memory(b: &mut test::Bencher) {
    b.iter(|| {
        let memory = allocate_agent_memory(4096);
        test::black_box(memory);
    });
}

#[bench]
fn bench_deallocate_memory(b: &mut test::Bencher) {
    let memory = allocate_agent_memory(4096).unwrap();
    b.iter(|| {
        deallocate_agent_memory(memory, 4096);
    });
}

