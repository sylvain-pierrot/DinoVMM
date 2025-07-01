use vmm::VMM;

fn main() {
    let mem_size = 0x4000;
    let guest_addr = 0x1000;
    let asm_code: &[u8];

    #[cfg(target_arch = "x86_64")]
    {
        asm_code = &[
            0xba, 0xf8, 0x03, /* mov $0x3f8, %dx */
            0x00, 0xd8, /* add %bl, %al */
            0x04, b'0', /* add $'0', %al */
            0xee, /* out %al, %dx */
            0xec, /* in %dx, %al */
            0xc6, 0x06, 0x00, 0x80,
            0x00, /* movl $0, (0x8000); This generates a MMIO Write. */
            0x8a, 0x16, 0x00, 0x80, /* movl (0x8000), %dl; This generates a MMIO Read. */
            0xf4, /* hlt */
        ];
    }

    let mut vmm = VMM::new(mem_size, guest_addr);
    vmm.configure();
    vmm.load_code(asm_code);
    vmm.run();
}
