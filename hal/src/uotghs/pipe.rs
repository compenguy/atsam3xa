use crate::target_device::uotghs::hstpipcfg::{PBK_A, PTOKEN_A, PTYPE_A};
use crate::target_device::UOTGHS;

// USB On-The-Go Interface RAM base address
const UOTGHS_RAM_SIZE: usize = 0x8000;
const UOTGHS_RAM_ADDR: usize = 0x2018_0000;

pub(crate) const MAX_PIPES: u8 = 10;

/// Errors that can result from operations on pipes
#[derive(Debug, Clone, Copy)]
pub enum PipeError {
    /// The referenced pipe does not exist.
    OutOfRange(u8),
    /// The specified size for this pipe is invalid.
    InvalidSize(u16),
    /// The provided pipe configuration was rejected.
    InvalidConfiguration(u32),
    /// No more pipes are available for allocation.
    OutOfPipes,
    /// The pipe requested for this operation is invalid.
    InvalidOperation,
}

pub struct Pipe<'a> {
    uotghs_p: &'a mut UOTGHS,
    pipe_num: u8,
}

impl<'a> Pipe<'a> {
    pub fn get(uotghs_p: &'a mut UOTGHS, pipe_num: u8) -> Result<Self, PipeError> {
        if pipe_num < MAX_PIPES {
            Ok(Self { uotghs_p, pipe_num })
        } else {
            Err(PipeError::OutOfRange(pipe_num))
        }
    }

    pub fn alloc(
        uotghs_p: &'a mut UOTGHS,
        address: u8,
        ep_num: u8,
        ep_type: PTYPE_A,
        ep_dir: PTOKEN_A,
        ep_size: u16,
        poll_freq: u8,
        num_banks: PBK_A,
    ) -> Result<Self, PipeError> {
        for pipe_num in 1..MAX_PIPES {
            let pipe = Self { uotghs_p, pipe_num };

            if pipe.enabled() {
                continue;
            }

            pipe.init_n(
                address, ep_num, ep_type, ep_dir, ep_size, poll_freq, num_banks,
            )?;
            return Ok(pipe);
        }

        Err(PipeError::OutOfPipes)
    }

    pub fn init_0(&mut self, address: u8, ep_size: u16) -> Result<(), PipeError> {
        if ep_size < 8 {
            return Err(PipeError::InvalidSize(ep_size));
        }

        if self.enabled() {
            return Ok(());
        }

        self.enable();

        // Pipe sizes are powers of 2, so we find the smallest power of two that
        // the ep size fits in.  The enum of valid values assigns 0 to an 8-byte
        // pipe, 1 to a 16-byte pipe, etc.  Consequently, we can count the
        // number of trailing zeroes on that power of 2 and subtract four to get
        // the correct enum value.
        //  8 bytes = 0
        // 16 bytes = 1
        let pipe_size: u8 = (ep_size.next_power_of_two().trailing_zeros() - 4) as u8;
        self.uotghs_p.hstpipcfg()[0].modify(|_, w| unsafe {
            w.intfrq()
                .bits(0)
                .pepnum()
                .bits(0)
                .ptype()
                .ctrl()
                .ptoken()
                .setup()
                .psize()
                .bits(pipe_size)
                .pbk()
                ._1_bank()
                .autosw()
                .clear_bit()
        });
        self.uotghs_p.hstpipcfg()[0].modify(|_, w| w.alloc().set_bit());

        if self.uotghs_p.hstpipisr()[0].read().cfgok().bit_is_clear() {
            self.uotghs_p.hstpip.modify(|_, w| w.pen0().clear_bit());
            return Err(PipeError::InvalidConfiguration(
                self.uotghs_p.hstpipcfg()[0].read().bits(),
            ));
        }

        self.configure_address(address);
        Ok(())
    }

    pub(crate) fn init_n(
        &mut self,
        address: u8,
        ep_num: u8,
        ep_type: PTYPE_A,
        ep_dir: PTOKEN_A,
        ep_size: u16,
        poll_freq: u8,
        num_banks: PBK_A,
    ) -> Result<(), PipeError> {
        self.enable();

        // Pipe sizes are powers of 2, so we find the smallest power of two that
        // the ep size fits in.  The enum of valid values assigns 0 to an 8-byte
        // pipe, 1 to a 16-byte pipe, etc.  Consequently, we can count the
        // number of trailing zeroes on that power of 2 and subtract four to get
        // the correct enum value.
        //  8 bytes = 0
        // 16 bytes = 1
        let pipe_size: u8 = (ep_size.next_power_of_two().trailing_zeros() - 4) as u8;
        self.uotghs_p.hstpipcfg()[self.pipe_num as usize].modify(|_, w| unsafe {
            w.intfrq()
                .bits(poll_freq)
                .pepnum()
                .bits(ep_num)
                .ptype()
                .variant(ep_type)
                .ptoken()
                .variant(ep_dir)
                .psize()
                .bits(pipe_size)
                .pbk()
                .variant(num_banks)
                .autosw()
                .set_bit()
        });
        self.uotghs_p.hstpipcfg()[self.pipe_num as usize].modify(|_, w| w.alloc().set_bit());

        if self.uotghs_p.hstpipisr()[self.pipe_num as usize]
            .read()
            .cfgok()
            .bit_is_clear()
        {
            self.disable();
            return Err(PipeError::InvalidConfiguration(
                self.uotghs_p.hstpipcfg()[self.pipe_num as usize]
                    .read()
                    .bits(),
            ));
        }

        self.configure_address(address);
        Ok(())
    }

    pub fn free(&mut self) {
        self.disable();
        self.uotghs_p.hstpipcfg()[self.pipe_num as usize].modify(|_, w| w.alloc().clear_bit());
        // forcibly reset the pipe
        self.enable();
        self.disable();
    }

    pub fn enabled(&mut self) -> bool {
        match self.pipe_num {
            // TODO: is the documentation off by one because pipe 0 can't be en/disabled?
            // i.e. 1=>pen0, 2=>pen1, etc
            0 => self.uotghs_p.hstpip.read().pen0().bit_is_set(),
            1 => self.uotghs_p.hstpip.read().pen1().bit_is_set(),
            2 => self.uotghs_p.hstpip.read().pen2().bit_is_set(),
            3 => self.uotghs_p.hstpip.read().pen3().bit_is_set(),
            4 => self.uotghs_p.hstpip.read().pen4().bit_is_set(),
            5 => self.uotghs_p.hstpip.read().pen5().bit_is_set(),
            6 => self.uotghs_p.hstpip.read().pen6().bit_is_set(),
            7 => self.uotghs_p.hstpip.read().pen7().bit_is_set(),
            8 => self.uotghs_p.hstpip.read().pen8().bit_is_set(),
            //9 => self.uotghs_p.hstpip.read().pen9().bit_is_set(),
            _ => unreachable!(),
        }
    }

    pub fn enable(&mut self) {
        match self.pipe_num {
            // TODO: is the documentation off by one because pipe 0 can't be en/disabled?
            // i.e. 1=>pen0, 2=>pen1, etc
            0 => self.uotghs_p.hstpip.modify(|_, w| w.pen0().set_bit()),
            1 => self.uotghs_p.hstpip.modify(|_, w| w.pen1().set_bit()),
            2 => self.uotghs_p.hstpip.modify(|_, w| w.pen2().set_bit()),
            3 => self.uotghs_p.hstpip.modify(|_, w| w.pen3().set_bit()),
            4 => self.uotghs_p.hstpip.modify(|_, w| w.pen4().set_bit()),
            5 => self.uotghs_p.hstpip.modify(|_, w| w.pen5().set_bit()),
            6 => self.uotghs_p.hstpip.modify(|_, w| w.pen6().set_bit()),
            7 => self.uotghs_p.hstpip.modify(|_, w| w.pen7().set_bit()),
            8 => self.uotghs_p.hstpip.modify(|_, w| w.pen8().set_bit()),
            //9 => self.uotghs_p.hstpip.modify(|_, w| w.pen9().set_bit()),
            _ => unreachable!(),
        }
    }

    pub fn disable(&mut self) {
        match self.pipe_num {
            // TODO: is the documentation off by one because pipe 0 can't be en/disabled?
            // i.e. 1=>pen0, 2=>pen1, etc
            0 => self.uotghs_p.hstpip.modify(|_, w| w.pen0().clear_bit()),
            1 => self.uotghs_p.hstpip.modify(|_, w| w.pen1().clear_bit()),
            2 => self.uotghs_p.hstpip.modify(|_, w| w.pen2().clear_bit()),
            3 => self.uotghs_p.hstpip.modify(|_, w| w.pen3().clear_bit()),
            4 => self.uotghs_p.hstpip.modify(|_, w| w.pen4().clear_bit()),
            5 => self.uotghs_p.hstpip.modify(|_, w| w.pen5().clear_bit()),
            6 => self.uotghs_p.hstpip.modify(|_, w| w.pen6().clear_bit()),
            7 => self.uotghs_p.hstpip.modify(|_, w| w.pen7().clear_bit()),
            8 => self.uotghs_p.hstpip.modify(|_, w| w.pen8().clear_bit()),
            //9 => self.uotghs_p.hstpip.modify(|_, w| w.pen9().clear_bit()),
            _ => unreachable!(),
        }
    }

    pub fn reset(&mut self) {
        // Reset pipe data toggle for each pipe's banks
        self.uotghs_p.hstpipier_mut()[self.pipe_num as usize]
            .write_with_zero(|w| w.rstdts().set_bit());

        // Reset pipes - note this doesn't reset the pipe configuration, nor
        // does it disable the pipe.
        match self.pipe_num {
            // TODO: is the documentation off by one because pipe 0 can't be en/disabled?
            // i.e. 1=>pen0, 2=>pen1, etc
            0 => self.uotghs_p.hstpip.modify(|_, w| w.prst0().clear_bit()),
            1 => self.uotghs_p.hstpip.modify(|_, w| w.prst1().clear_bit()),
            2 => self.uotghs_p.hstpip.modify(|_, w| w.prst2().clear_bit()),
            3 => self.uotghs_p.hstpip.modify(|_, w| w.prst3().clear_bit()),
            4 => self.uotghs_p.hstpip.modify(|_, w| w.prst4().clear_bit()),
            5 => self.uotghs_p.hstpip.modify(|_, w| w.prst5().clear_bit()),
            6 => self.uotghs_p.hstpip.modify(|_, w| w.prst6().clear_bit()),
            7 => self.uotghs_p.hstpip.modify(|_, w| w.prst7().clear_bit()),
            8 => self.uotghs_p.hstpip.modify(|_, w| w.prst8().clear_bit()),
            //9 => self.uotghs_p.hstpip.modify(|_, w| w.prst9().clear_bit()),
            _ => unreachable!(),
        }
    }

    fn configure_address(&mut self, address: u8) {
        match self.pipe_num {
            0 => self
                .uotghs_p
                .hstaddr1
                .modify(|_, w| unsafe { w.hstaddrp0().bits(address) }),
            1 => self
                .uotghs_p
                .hstaddr1
                .modify(|_, w| unsafe { w.hstaddrp1().bits(address) }),
            2 => self
                .uotghs_p
                .hstaddr1
                .modify(|_, w| unsafe { w.hstaddrp2().bits(address) }),
            3 => self
                .uotghs_p
                .hstaddr1
                .modify(|_, w| unsafe { w.hstaddrp3().bits(address) }),
            4 => self
                .uotghs_p
                .hstaddr2
                .modify(|_, w| unsafe { w.hstaddrp4().bits(address) }),
            5 => self
                .uotghs_p
                .hstaddr2
                .modify(|_, w| unsafe { w.hstaddrp5().bits(address) }),
            6 => self
                .uotghs_p
                .hstaddr2
                .modify(|_, w| unsafe { w.hstaddrp6().bits(address) }),
            7 => self
                .uotghs_p
                .hstaddr2
                .modify(|_, w| unsafe { w.hstaddrp7().bits(address) }),
            8 => self
                .uotghs_p
                .hstaddr3
                .modify(|_, w| unsafe { w.hstaddrp8().bits(address) }),
            9 => self
                .uotghs_p
                .hstaddr3
                .modify(|_, w| unsafe { w.hstaddrp9().bits(address) }),
            _ => unreachable!(),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) {
        let _len = self.uotghs_p.hstpipisr()[self.pipe_num as usize]
            .read()
            .pbyct();

        // The uotghs ram is divided into a segment for each pipe, where each
        // segment is the size of a bus transfer. So we turn it into a slice of
        // transfers, and get the correct slice index for our pipe, and return
        // it as a pointer to the transfer data.
        let uotghs_ram =
            unsafe { core::slice::from_raw_parts(UOTGHS_RAM_ADDR as *const u8, UOTGHS_RAM_SIZE) };
        let pipe_ram = &uotghs_ram[self.pipe_num as usize..(self.pipe_num as usize + buf.len())];

        buf.copy_from_slice(pipe_ram);
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), PipeError> {
        if !self.enabled() {
            return Err(PipeError::InvalidOperation);
        }
        let _len = self.uotghs_p.hstpipisr()[self.pipe_num as usize]
            .read()
            .pbyct();

        // The uotghs ram is divided into a segment for each pipe, where each
        // segment is the size of a bus transfer. So we turn it into a slice of
        // transfers, and get the correct slice index for our pipe, and return
        // it as a pointer to the transfer data.
        let uotghs_ram =
            unsafe { core::slice::from_raw_parts_mut(UOTGHS_RAM_ADDR as *mut u8, UOTGHS_RAM_SIZE) };
        let pipe_ram =
            &mut uotghs_ram[self.pipe_num as usize..(self.pipe_num as usize + buf.len())];

        pipe_ram.copy_from_slice(buf);
        Ok(())
    }

    pub fn send(&mut self, token_type: PTOKEN_A) -> Result<(), PipeError> {
        if !self.enabled() {
            return Err(PipeError::InvalidOperation);
        }

        // Must first configure the pipe token
        self.uotghs_p.hstpipcfg()[self.pipe_num as usize]
            .modify(|_, w| w.ptoken().variant(token_type));

        // Clear interrupt flags
        self.uotghs_p.hstpipicr()[self.pipe_num as usize].write_with_zero(|w| {
            w.txstpic()
                .set_bit()
                .rxinic()
                .set_bit()
                .txoutic()
                .set_bit()
                .shortpacketic()
                .set_bit()
                .nakedic()
                .set_bit()
        });

        // Send packet
        self.uotghs_p.hstpipidr()[self.pipe_num as usize]
            .write_with_zero(|w| w.fifoconc().set_bit().pfreezec().set_bit());
        Ok(())
    }

    pub fn is_transfer_complete(&mut self, token_type: PTOKEN_A) -> bool {
        match token_type {
            PTOKEN_A::SETUP => {
                if self.uotghs_p.hstpipisr()[self.pipe_num as usize]
                    .read()
                    .txstpi()
                    .bit_is_set()
                {
                    self.uotghs_p.hstpipier()[self.pipe_num as usize]
                        .write_with_zero(|w| w.pfreezes().set_bit());
                    self.uotghs_p.hstpipicr()[self.pipe_num as usize]
                        .write_with_zero(|w| w.txstpic().set_bit());
                    return true;
                }
            }
            PTOKEN_A::IN => {
                if self.uotghs_p.hstpipisr()[self.pipe_num as usize]
                    .read()
                    .rxini()
                    .bit_is_set()
                {
                    // In case of low USB speed and with a high CPU frequency,
                    // a ACK from host can be always running on USB line
                    // then wait end of ACK on IN pipe.
                    while self.uotghs_p.hstpipimr()[self.pipe_num as usize]
                        .read()
                        .pfreeze()
                        .bit_is_clear()
                    {}
                    self.uotghs_p.hstpipicr()[self.pipe_num as usize]
                        .write_with_zero(|w| w.rxinic().set_bit());
                    return true;
                }
            }
            PTOKEN_A::OUT => {
                if self.uotghs_p.hstpipisr()[self.pipe_num as usize]
                    .read()
                    .txouti()
                    .bit_is_set()
                {
                    self.uotghs_p.hstpipier()[self.pipe_num as usize]
                        .write_with_zero(|w| w.pfreezes().set_bit());
                    self.uotghs_p.hstpipicr()[self.pipe_num as usize]
                        .write_with_zero(|w| w.txoutic().set_bit());
                    return true;
                }
            }
        }
        false
    }
}
