//! IPC benchmarks

#![no_std]
#![feature(test)]

extern crate test;

use aios_kernel_ipc::{BinaryIPC, message::IPCMessage};

#[bench]
fn bench_send_message(b: &mut test::Bencher) {
    let ipc = BinaryIPC::new();
    let message = IPCMessage::new(1, 2, vec![0u8; 1024], vec![]);
    
    b.iter(|| {
        let _ = ipc.send(message.clone());
        test::black_box(());
    });
}

