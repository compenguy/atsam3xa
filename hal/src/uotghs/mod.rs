use crate::clock::SystemClocks;
use crate::gpio::{Pb10, Pb11, PfA};
use crate::target_device::generic::Variant;
use crate::target_device::uotghs::sr::SPEED_A;
use crate::target_device::UOTGHS;
use starb::{Reader, RingBuffer, Writer};
use usb_host::{
    DescriptorType, DeviceDescriptor, Direction, Driver, DriverError, Endpoint, RequestCode,
    RequestDirection, RequestKind, RequestRecipient, RequestType, TransferError, TransferType,
    USBHost, WValue,
};

pub mod pipe;
use crate::uotghs::pipe::{Pipe, MAX_PIPES};

// TODO: verify this number
const MAX_DEVICES: usize = 4;
const NAK_LIMIT: usize = 15;

/// Errors that can result from host operations
pub enum HostError {
    OutOfDevices,
    DriverError(DriverError),
}

/// Models the Host state of the UOTGHS controller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HostState {
    /// As Host, Vbus must first be powered on, before any devices are detected.
    NoVbus,
    /// No attached device.
    Detached,
    /// Device attached, device is in `TaskState`.
    Attached(TaskState),
    /// Host controller is in an invalid state, or an error was reported.
    Error,
}

/// Models the state of an attached device to the UOTGHS controller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TaskState {
    /// Device is not yet fully configured.
    Configuring,
    /// Device is active.
    Running,
    /// Device is in an invalid state, or an error was reported.
    Error,
}

type Events = RingBuffer<HostState>;
type EventReader = Reader<'static, HostState>;
type EventWriter = Writer<'static, HostState>;

static mut EVENTS: Events = Events::new();

// This is the primary control endpoint, and isn't dynamic or configurable
// so we can just hardcode a bunch of values and pretty much no behavior for it.
struct Addr0EP0 {
    max_packet_size: u16,
    in_toggle: bool,
    out_toggle: bool,
}

impl Addr0EP0 {
    fn new(max_packet_size: u16) -> Self {
        Self {
            max_packet_size,
            in_toggle: true,
            out_toggle: true,
        }
    }
}

impl Endpoint for Addr0EP0 {
    fn address(&self) -> u8 {
        0
    }

    fn endpoint_num(&self) -> u8 {
        0
    }

    fn transfer_type(&self) -> TransferType {
        TransferType::Control
    }

    fn direction(&self) -> Direction {
        Direction::In
    }

    fn max_packet_size(&self) -> u16 {
        self.max_packet_size
    }

    fn in_toggle(&self) -> bool {
        self.in_toggle
    }

    fn set_in_toggle(&mut self, toggle: bool) {
        self.in_toggle = toggle;
    }

    fn out_toggle(&self) -> bool {
        self.out_toggle
    }

    fn set_out_toggle(&mut self, toggle: bool) {
        self.out_toggle = toggle;
    }
}

fn handler(usbp: usize, events: &mut EventWriter) {
    let uotghs_p: &mut UOTGHS = unsafe { core::mem::transmute(usbp) };

    let imask = uotghs_p.hstimr.read();
    let iflags = uotghs_p.hstisr.read();

    // Manage disconnect event
    if iflags.ddisci().bit_is_set() && imask.ddiscie().bit_is_set() {
        // Clear disconnect interrupt and disable it
        uotghs_p.hsticr.write_with_zero(|w| w.ddiscic().set_bit());
        uotghs_p.hstidr.write_with_zero(|w| w.ddisciec().set_bit());

        // stop reset signal, in case of disconnection during reset
        uotghs_p.hstctrl.modify(|_, w| w.reset().clear_bit());

        // TODO: disable wakeup/resume interrupts in case of disconnect during
        // suspend mode
        // self.uotghs_p.hstidr.write_with_zero(|w| w.hwupiec().set_bit()
        //                                           .rsmediec().set_bit()
        //                                           .rxrsmiec().set_bit()

        // Clear connect interrupt and enable it
        uotghs_p.hsticr.write_with_zero(|w| w.dconnic().set_bit());
        uotghs_p.hstier.write_with_zero(|w| w.dconnies().set_bit());

        // Update state
        events.unshift(HostState::Detached).unwrap();
    }

    // Manage connect event
    if iflags.dconni().bit_is_set() && imask.dconnie().bit_is_set() {
        // Clear connect interrupt and disable it
        uotghs_p.hsticr.write_with_zero(|w| w.dconnic().set_bit());
        uotghs_p.hstidr.write_with_zero(|w| w.dconniec().set_bit());

        // Clear disconnect interrupt and enable it
        uotghs_p.hsticr.write_with_zero(|w| w.ddiscic().set_bit());
        uotghs_p.hstier.write_with_zero(|w| w.ddiscies().set_bit());

        // Update state
        events
            .unshift(HostState::Attached(TaskState::Configuring))
            .unwrap();
    }

    let gflags = uotghs_p.sr.read();
    // Manage vbus error
    if gflags.vberri().bit_is_set() {
        // Clear vbus error interrupt
        uotghs_p.scr.write_with_zero(|w| w.vberric().set_bit());

        // Update state
        events.unshift(HostState::Detached).unwrap();
    }

    // Wait for usb clock to be ready after interrupt
    while gflags.clkusable().bit_is_clear() {}

    uotghs_p.ctrl.modify(|_, w| w.frzclk().clear_bit());

    // Manage vbus state transition
    if gflags.vbusti().bit_is_set() {
        // Clear vbus transition interrupt
        uotghs_p.scr.write_with_zero(|w| w.vbustic().set_bit());

        // Update state
        if gflags.vbus().bit_is_set() {
            events.unshift(HostState::Detached).unwrap();
        } else {
            events.unshift(HostState::NoVbus).unwrap();
        }
    }

    // Squelch other errors
    let err_flags = uotghs_p.ctrl.read();
    if err_flags.vberre().bit_is_set()
        || err_flags.bcerre().bit_is_set()
        || err_flags.hnperre().bit_is_set()
        || err_flags.stoe().bit_is_set()
    {
        uotghs_p.scr.write_with_zero(|w| {
            w.vberric()
                .set_bit()
                .bcerric()
                .set_bit()
                .hnperric()
                .set_bit()
                .stoic()
                .set_bit()
        });
    }
}

use core::mem::{self, MaybeUninit};
use core::ptr;

struct Device {
    addr: u8,
}

struct DeviceTable {
    tbl: [Option<Device>; MAX_DEVICES],
}

impl DeviceTable {
    fn new() -> Self {
        let tbl = {
            let mut devs: [MaybeUninit<Option<Device>>; MAX_DEVICES] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for d in &mut devs[..] {
                unsafe { ptr::write(d.as_mut_ptr(), None) }
            }
            unsafe { mem::transmute(devs) }
        };

        Self { tbl }
    }

    /// Allocate a device with the next available address.
    fn next(&mut self) -> Option<&mut Device> {
        for (i, dev) in self.tbl.iter_mut().enumerate().find(|(_, x)| x.is_none()) {
            dev.replace(Device { addr: i as u8 });
            return dev.as_mut();
        }
        None
    }

    /// Remove the device at address `addr`.
    fn remove(&mut self, addr: u8) -> Option<Device> {
        self.tbl[addr as usize].take()
    }
}

pub struct UsbOtgHs {
    uotghs_p: UOTGHS,

    uotg_id: Option<Pb11<PfA>>,
    uotg_vbof: Option<Pb10<PfA>>,

    events: EventReader,
    host_state: HostState,

    devices: DeviceTable,
}

impl UsbOtgHs {
    pub fn new(uotghs_p: UOTGHS, uotg_id: Option<Pb11<PfA>>, uotg_vbof: Option<Pb10<PfA>>) -> Self {
        let (eventr, _) = unsafe { EVENTS.split() };
        Self {
            uotghs_p,
            uotg_id,
            uotg_vbof,
            events: eventr,
            host_state: HostState::NoVbus,
            devices: DeviceTable::new(),
        }
    }

    pub fn get_interrupt_handler(&self) -> impl FnMut() {
        let (_, mut eventw) = unsafe { EVENTS.split() };
        let usbp = &self.uotghs_p as *const _ as usize;
        move || handler(usbp, &mut eventw)
    }

    pub fn reset(&mut self) {
        // Save current bus speed configuration so we can restore it
        // after reset
        let spdconf = self.uotghs_p.devctrl.read().spdconf().variant();

        // disables transceiver, clock inputs, registers restored to reset
        // values.
        self.uotghs_p.ctrl.modify(|_, w| w.usbe().clear_bit());
        self.uotghs_p.ctrl.modify(|_, w| w.usbe().set_bit());

        // flush the event buffer
        while self.events.shift().is_some() {}

        // Restore bus speed configuration (PMC clock configurations should
        // still be properly configured).
        self.uotghs_p
            .devctrl
            .modify(|_, w| w.spdconf().variant(spdconf));

        for pipe_num in 0..MAX_PIPES {
            // unwrap here is safe (except against internal structural changes)
            // because we're bounding the get by MAX_PIPES
            Pipe::get(&mut self.uotghs_p, pipe_num).unwrap().reset();
        }

        // Disable OTG host/device switch, force host mode
        self.uotghs_p
            .ctrl
            .modify(|_, w| w.uide().clear_bit().uimod().clear_bit());

        // According to the Arduino Due circuit the VBOF must be active high to
        // power up the remote device
        self.uotghs_p.ctrl.modify(|_, w| w.vbuspo().set_bit());

        self.uotghs_p.ctrl.modify(|_, w| w.otgpade().set_bit());
        self.uotghs_p.ctrl.modify(|_, w| w.frzclk().clear_bit());

        // Wait for usb clock to be ready after unfreezing
        while self.uotghs_p.sr.read().clkusable().bit_is_clear() {}

        cortex_m::interrupt::free(|cs| self.init(cs));
    }

    fn init(&mut self, _cs: &cortex_m::interrupt::CriticalSection) {
        // Clear any lingering raised interrupts from previous activations
        self.uotghs_p.hsticr.write_with_zero(|w| {
            w.dconnic()
                .set_bit()
                .ddiscic()
                .set_bit()
                .hsofic()
                .set_bit()
                .hwupic()
                .set_bit()
                .rsmedic()
                .set_bit()
                .rstic()
                .set_bit()
                .rxrsmic()
                .set_bit()
        });

        // Clear vbus transition interrupt
        self.uotghs_p.scr.write_with_zero(|w| w.vbustic().set_bit());

        // Enable Vbus interrupts, disable automatic vbus control after error
        self.uotghs_p
            .ctrl
            .modify(|_, w| w.vbushwc().set_bit().vbuste().set_bit().vberre().set_bit());

        // Enable vbus
        self.uotghs_p.sfr.write_with_zero(|w| w.vbusrqs().set_bit());

        // Force Vbus interrupt when Vbus is always high
        // This is possible due to a short timing between a Host mode stop/start.
        if self.uotghs_p.sr.read().vbus().bit_is_set() {
            self.uotghs_p.sfr.write_with_zero(|w| w.vbustis().set_bit());
        }

        // enable dconn interrupt
        self.uotghs_p
            .hstier
            .write_with_zero(|w| w.dconnies().set_bit());

        self.uotghs_p.ctrl.modify(|_, w| w.frzclk().set_bit());

        self.host_state = HostState::NoVbus;
    }

    pub fn set_high_speed(&mut self, clocks: &mut SystemClocks) {
        // configure USB peripheral to Normal mode (480M, High speed)
        self.uotghs_p.devctrl.write(|w| w.spdconf().normal());

        // Enable usb clocks for high speed mode
        clocks.enable_usb_high_speed_clocks();
    }

    pub fn set_full_speed(&mut self, clocks: &mut SystemClocks) {
        // configure USB peripheral for Low Power mode (48M, Full speed)
        self.uotghs_p.devctrl.write(|w| w.spdconf().low_power());

        // Enable usb clocks for full speed mode
        clocks.enable_usb_high_speed_clocks();
    }

    pub fn task(&mut self, drivers: &mut [&mut dyn Driver]) {
        if let Some(event) = self.events.shift() {
            self.host_state = event;
        }

        match self.host_state {
            HostState::Attached(TaskState::Configuring) => {
                self.host_state = match self.configure_dev(drivers) {
                    Ok(_) => HostState::Attached(TaskState::Running),
                    Err(e) => HostState::Attached(TaskState::Error),
                }
            }
            HostState::Attached(TaskState::Running) => {
                // TODO: find some way to query a monotonic clock
                let jiffies = 0;
                for d in &mut drivers[..] {
                    if let Err(e) = d.tick(jiffies, self) {
                        if let DriverError::Permanent(a, _) = e {
                            d.remove_device(a);
                            self.devices.remove(a);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn configure_dev(&mut self, drivers: &mut [&mut dyn Driver]) -> Result<(), HostError> {
        let none: Option<&mut [u8]> = None;
        let max_packet_size: u16 = match self.uotghs_p.sr.read().speed().variant() {
            Variant::Val(SPEED_A::HIGH_SPEED) => 512,
            Variant::Val(SPEED_A::FULL_SPEED) => 64,
            Variant::Val(SPEED_A::LOW_SPEED) => 8,
            _ => unreachable!(),
        };
        let mut a0ep0 = Addr0EP0::new(max_packet_size);
        let mut dev_desc: DeviceDescriptor =
            unsafe { MaybeUninit::<DeviceDescriptor>::uninit().assume_init() };
        self.control_transfer(
            &mut a0ep0,
            RequestType::from((
                RequestDirection::DeviceToHost,
                RequestKind::Standard,
                RequestRecipient::Device,
            )),
            RequestCode::GetDescriptor,
            WValue::from((0, DescriptorType::Device as u8)),
            0,
            Some(unsafe { to_slice_mut(&mut dev_desc) }),
        )?;

        let addr = self.devices.next().ok_or(HostError::OutOfDevices)?.addr;
        self.control_transfer(
            &mut a0ep0,
            RequestType::from((
                RequestDirection::HostToDevice,
                RequestKind::Standard,
                RequestRecipient::Device,
            )),
            RequestCode::SetAddress,
            WValue::from((addr, 0)),
            0,
            none,
        )?;

        // Now that the device is addressed, see if any drivers want
        // it.
        for d in &mut drivers[..] {
            if d.want_device(&dev_desc) {
                return d
                    .add_device(dev_desc, addr)
                    .map_err(|e| HostError::DriverError(e));
            }
        }
        Ok(())
    }
}

impl USBHost for UsbOtgHs {
    fn control_transfer(
        &mut self,
        ep: &mut dyn Endpoint,
        bm_request_type: RequestType,
        b_request: RequestCode,
        w_value: WValue,
        w_index: u16,
        buf: Option<&mut [u8]>,
    ) -> Result<usize, TransferError> {
        // TODO: need support for mapping endpoints to pipes
        let mut pipe = self.pipe_table.pipe_for(ep);
        // TODO: actually issue the request on the specified pipe
        let len = pipe.control_transfer(ep, bm_request_type, b_request, w_value, w_index, buf)?;
        Ok(len)
    }

    fn in_transfer(
        &mut self,
        ep: &mut dyn Endpoint,
        buf: &mut [u8],
    ) -> Result<usize, TransferError> {
        // TODO: need support for mapping endpoints to pipes
        let mut pipe = self.pipe_table.pipe_for(ep);
        // TODO: actually transfer from the specified pipe into the buffer
        let len = pipe.in_transfer(ep, buf, NAK_LIMIT)?;
        Ok(len)
    }

    fn out_transfer(&mut self, ep: &mut dyn Endpoint, buf: &[u8]) -> Result<usize, TransferError> {
        // TODO: need support for mapping endpoint numbers to pipe numbers
        let mut pipe = self.pipe_table.pipe_for(ep);
        // TODO: actually transfer the buffer out on the specified pipe
        let len = pipe.out_transfer(ep, buf, NAK_LIMIT)?;
        Ok(len)
    }
}

unsafe fn to_slice_mut<T>(v: &mut T) -> &mut [u8] {
    let ptr = v as *mut T as *mut u8;
    let len = mem::size_of::<T>();
    core::slice::from_raw_parts_mut(ptr, len)
}
