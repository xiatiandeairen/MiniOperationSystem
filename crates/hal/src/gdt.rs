//! Global Descriptor Table (GDT) and Task State Segment (TSS) setup.
//!
//! Configures a minimal GDT with kernel code/data segments and a TSS
//! that provides an Interrupt Stack Table entry for the double-fault handler.

use spin::Lazy;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// IST index used for the double-fault handler stack.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Size of the double-fault handler stack in bytes (20 KiB).
const STACK_SIZE: usize = 4096 * 5;

/// Static stack memory for the IST double-fault entry.
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

/// Task State Segment with an IST entry for double faults.
static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        // SAFETY: STACK is only accessed here during one-time lazy
        // initialisation, well before interrupts are enabled.
        let stack_start = VirtAddr::from_ptr(core::ptr::addr_of!(STACK));
        stack_start + STACK_SIZE as u64
    };
    tss
});

/// Selectors produced when the GDT is built.
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// The GDT together with the selectors needed at load time.
static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
    (
        gdt,
        Selectors {
            code_selector,
            data_selector,
            tss_selector,
        },
    )
});

/// Loads the GDT and sets the code-segment, data-segment, and TSS selectors.
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    // SAFETY: Selectors were produced from the GDT we just loaded.
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
