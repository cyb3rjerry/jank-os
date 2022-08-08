// main.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(jank_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use jank_os::println;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Page;
use core::panic::PanicInfo;

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
    use jank_os::memory;
    use x86_64::{VirtAddr};

    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};


    println!("Hello World{}", "!");
    jank_os::init();

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