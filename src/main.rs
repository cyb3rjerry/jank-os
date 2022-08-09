// main.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(jank_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use jank_os::println;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

entry_point!(kernel_main);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use jank_os::allocator;
    use jank_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::{VirtAddr};

    println!("Hello World{}", "!");
    jank_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // ===============================================
    // create heap value
    let heap_value = Box::new(41);
    println!("heap_value at: {:p}", heap_value);

    // create dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create ref counter vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1,2,3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));
    // ===============================================

    #[cfg(test)]
    test_main();

    println!("We're alive!");
    jank_os::hlt_loop();
}

// Called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    jank_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    jank_os::test_panic_handler(info)
}


#[test_case]
fn trivial_assertion() {
    assert_eq!(1,1);
}