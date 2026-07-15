use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use crate::println;
use crate::print;
use crate::gdt;
use spin;
use pic8259::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Safe mutable access
// Unsafe because ChainedPics::new can cause UB with wrong offsets.
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

// Timer uses line 0 of primary PIC, so we make a C-like enum
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// We don't want to use a static mut instance of the IDT, since it would require an unsafe block on each access
// Mutation only happens once, on initialization
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
                // this is the index of the TSS to the double fault stack
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}

// CPU reacts similarly to exceptions and external interrupts, hence the identical function signatures
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    print!(".");

    /* Figures out which of the PICs sent the interrupt
    Uses the command and data ports to send an End of Interrupt signal
    If the secondary PIC sent the interrupt, both PICs must be notified
    The secondary PIC is connected to an input of the Primary PIC
    Unsafe function because using the wrong interrupt vector number can lead to system hang or deleting an important unsent interrupt
    This uses the PIT (Programmable Interval Timer)
    */
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    static KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        spin::Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        ));

    let mut keyboard = KEYBOARD.lock();
    /*
    We must read the scancode of the pressed key to allow the keyboard controller to send another interrupt
    We must read from the data port of the PS/2 controller, which is the I/O port with the number 0x60
    */
    let mut port = Port::new(0x60); 
    let scancode: u8 = unsafe { port.read() };

    /*
    PS/2 keyboards emulate scancode set 1
    The lower 7 bits of a scancode byte define the key
    The most significant bit defines whether it's a press or a release
    Keys not present in the original IBM XT keyboard generate two scancodes in succession
    A 0xe0 escape byte and then a byte representing the key.
    */
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}