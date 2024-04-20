use crate::{Color, Image};
use embassy_stm32::gpio::*;
use embassy_stm32::peripherals::*;
use embassy_time::Ticker;
use embassy_time::Timer;

use embassy_time::Delay;
use embedded_hal::delay::DelayNs;

/*
Board	uC
SB	    PC5
LAT	    PC4
RST	    PC3
SCK	    PB1
SDA	    PA4
C0	    PB2
C1	    PA15
C2	    PA2
C3	    PA7
C4	    PA6
C5	    PA5
C6	    PB0
C7	    PA3
*/

pub struct Matrix<'a> {
    sb: Output<'a, PC5>,
    lat: Output<'a, PC4>,
    rst: Output<'a, PC3>,
    sck: Output<'a, PB1>,
    sda: Output<'a, PA4>,
    rows: [Output<'a, AnyPin>; 8],
}

impl Matrix<'_> {
    /// Create a new matrix from the control registers and the individual
    /// unconfigured pins. SB and LAT will be set high by default, while
    /// other pins will be set low. After 100ms, RST will be set high, and
    /// the bank 0 will be initialized by calling `init_bank0()` on the
    /// newly constructed structure.
    /// The pins will be set to very high speed mode.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        pa2: PA2,
        pa3: PA3,
        pa4: PA4,
        pa5: PA5,
        pa6: PA6,
        pa7: PA7,
        pa15: PA15, // <Alternate<PushPull, 0>>,
        pb0: PB0,
        pb1: PB1,
        pb2: PB2,
        pc3: PC3,
        pc4: PC4,
        pc5: PC5,
    ) -> Self {
        /*
        sb: Output<'a, PC5>,
        lat: Output<'a, PC4>,
        rst: Output<'a, PC3>,
        sck: Output<'a, PB1>,
        sda: Output<'a, PA4>,
        rows: [Output<'a, AnyPin>; 8],
        */

        let sb = Output::new(pc5, Level::High, Speed::VeryHigh);
        let lat = Output::new(pc4, Level::High, Speed::VeryHigh);
        let rst = Output::new(pc3, Level::Low, Speed::VeryHigh);
        let sck = Output::new(pb1, Level::Low, Speed::VeryHigh);
        let sda = Output::new(pa4, Level::Low, Speed::VeryHigh);
        let rows = [
            Output::new(pb2, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa15, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa2, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa7, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa6, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa5, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pb0, Level::Low, Speed::VeryHigh).degrade(),
            Output::new(pa3, Level::Low, Speed::VeryHigh).degrade(),
        ];

        let mut matrix = Self {
            sb,
            lat,
            rst,
            sck,
            sda,
            rows,
        };

        Timer::after_millis(100).await;
        matrix.rst.set_high();
        matrix.init_bank0(); // Chamada correta para init_bank0
        matrix
    }

    /// Make a brief high pulse of the SCK pin
    fn pulse_sck(&mut self) {
        self.sck.set_low();
        self.sck.set_high();
        self.sck.set_low();
    }

    /// Make a brief low pulse of the LAT pin
    fn pulse_lat(&mut self) {
        self.lat.set_high();
        self.lat.set_low();
        self.lat.set_high();
    }

    /// Send a byte on SDA starting with the MSB and pulse SCK high after each bit
    fn send_byte(&mut self, pixel: u8) {
        for i in (0..8).rev() {
            if pixel & (1 << i) != 0 {
                self.sda.set_high();
            } else {
                self.sda.set_low();
            }
            self.pulse_sck();
        }
    }

    /// Send a full row of bytes in BGR order and pulse LAT low. Gamma correction
    /// must be applied to every pixel before sending them. The previous row must
    /// be deactivated and the new one activated.
    pub fn send_row(&mut self, row: usize, pixels: &[Color]) {
        // Send the new row
        for pixel in pixels.iter().rev() {
            let corrected_pixel = pixel.gamma_correct();
            self.send_byte(corrected_pixel.b);
            self.send_byte(corrected_pixel.g);
            self.send_byte(corrected_pixel.r);
        }

        // Deactivate the previous row
        self.rows[(row + 7) % 8].set_low();

        Delay.delay_us(30);
        // Pulse LAT
        self.pulse_lat();

        // Activate the new row
        self.rows[row].set_high();
    }

    /// Initialize bank0 by temporarily setting SB to low and sending 144 one bits,
    /// pulsing SCK high after each bit and pulsing LAT low at the end. SB is then
    /// restored to high.
    fn init_bank0(&mut self) {
        self.sb.set_low();

        for _ in 0..144 {
            self.sda.set_high();
            self.pulse_sck();
        }
        self.pulse_lat();
        self.sb.set_high();
    }

    /// Display a full image, row by row, as fast as possible.
    pub async fn display_image(&mut self, image: &Image, ticker: &mut Ticker) {
        // Do not forget that image.row(n) gives access to the content of row n,
        // and that self.send_row() uses the same format.
        for row in 0..8 {
            ticker.next().await; // Wait for the next tick
            self.send_row(row, image.row(row));
        }
    }
}
