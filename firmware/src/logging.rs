use core::convert::Infallible;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use esp32c3_hal::ehal;

const BUF_CAPACITY: usize = 64;
static IS_TAKEN: AtomicBool = AtomicBool::new(false);

pub struct BufferedSerial {
    d: esp32c3_hal::UsbSerialJtag,
    buf: [u8; BUF_CAPACITY],
    buf_len: usize,
}
impl BufferedSerial {
    pub fn take() -> Option<BufferedSerial> {
        // TODO: Can this be weaker than SeqCst?
        if let Ok(false) =
            IS_TAKEN.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            Some(Self {
                d: esp32c3_hal::UsbSerialJtag,
                buf: [0; BUF_CAPACITY],
                buf_len: 0,
            })
        } else {
            None
        }
    }
}
impl ehal::serial::Write<u8> for BufferedSerial {
    type Error = Infallible;
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        if self.buf_len < BUF_CAPACITY {
            self.buf[self.buf_len] = word;
            self.buf_len += 1;
            Ok(())
        } else {
            self.flush()?;
            self.write(word)
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let slice = &self.buf[..self.buf_len - 1];
        // this is an evil hack and technically UB
        let str = unsafe { core::str::from_utf8_unchecked(slice) };
        self.d.write_str(str).unwrap();
        self.buf_len = 0;
        Ok(())
    }
}
