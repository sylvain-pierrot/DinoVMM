extern crate kvm_bindings;
extern crate kvm_ioctls;

use kvm_bindings::{kvm_userspace_memory_region, KVM_MEM_LOG_DIRTY_PAGES};
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
use std::ptr::null_mut;

pub struct VMM {
    kvm: Kvm,
    guest_addr: u64,
    host_addr: *mut u8,
    vm_fd: VmFd,
    vcpu_fd: VcpuFd,
}

impl VMM {
    pub fn new(mem_size: usize, guest_addr: u64) -> Self {
        let kvm = Kvm::new().expect("Failed to create KVM instance");
        let vm_fd = kvm.create_vm().expect("Failed to create VM");
        let host_addr: *mut u8 = unsafe {
            libc::mmap(
                null_mut(),
                mem_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
                -1,
                0,
            ) as *mut u8
        };

        let slot = 0;
        // When initializing the guest memory slot specify the
        // `KVM_MEM_LOG_DIRTY_PAGES` to enable the dirty log.
        let mem_region = kvm_userspace_memory_region {
            slot,
            guest_phys_addr: guest_addr,
            memory_size: mem_size as u64,
            userspace_addr: host_addr as u64,
            flags: KVM_MEM_LOG_DIRTY_PAGES,
        };

        unsafe { vm_fd.set_user_memory_region(mem_region).unwrap() };

        let vcpu_fd = vm_fd.create_vcpu(0).expect("Failed to create VCPU");

        VMM {
            kvm,
            guest_addr,
            host_addr,
            vm_fd,
            vcpu_fd,
        }
    }

    pub fn configure(&self) {
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 specific registry setup.
            let mut vcpu_sregs = self.vcpu_fd.get_sregs().unwrap();
            vcpu_sregs.cs.base = 0;
            vcpu_sregs.cs.selector = 0;
            self.vcpu_fd.set_sregs(&vcpu_sregs).unwrap();

            let mut vcpu_regs = self.vcpu_fd.get_regs().unwrap();
            vcpu_regs.rip = self.guest_addr;
            vcpu_regs.rax = 2;
            vcpu_regs.rbx = 3;
            vcpu_regs.rflags = 2;
            self.vcpu_fd.set_regs(&vcpu_regs).unwrap();
        }
    }

    pub fn load_code(&mut self, asm_code: &[u8]) {
        // Copy the assembly code into the guest memory.
        unsafe {
            std::ptr::copy_nonoverlapping(
                asm_code.as_ptr(),
                self.host_addr.add(self.guest_addr as usize),
                asm_code.len(),
            );
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.vcpu_fd.run().expect("run failed") {
                VcpuExit::IoIn(addr, data) => {
                    println!(
                        "Received an I/O in exit. Address: {:#x}. Data: {:#x}",
                        addr, data[0],
                    );
                }
                VcpuExit::IoOut(addr, data) => {
                    println!(
                        "Received an I/O out exit. Address: {:#x}. Data: {:#x}",
                        addr, data[0],
                    );
                }
                VcpuExit::MmioRead(addr, _data) => {
                    println!("Received an MMIO Read Request for the address {:#x}.", addr,);
                    // unsafe {
                    //     host_addr
                    //         .add(addr as usize)
                    //         .copy_to(data.as_mut_ptr(), data.len())
                    // };
                }
                VcpuExit::MmioWrite(addr, _data) => {
                    println!(
                        "Received an MMIO Write Request for the address {:#x}.",
                        addr,
                    );
                    // host_addr
                    //     .add(addr as usize)
                    //     .copy_from(data.as_ptr(), data.len());
                }
                VcpuExit::Hlt => {
                    break;
                }
                r => panic!("Unexpected exit reason: {:?}", r),
            }
        }
    }
}
