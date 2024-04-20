#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_stm32 as _; // Just to link it in the executable (it provides the vector table)
use panic_probe as _; // panic handler // link with the defmt-rtt library

use embassy_stm32::{gpio::*, peripherals::*};

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};

use embassy_stm32::bind_interrupts;
use embassy_stm32::dma::NoDma;
use embassy_stm32::rcc::*;
use embassy_stm32::Config;
use embassy_stm32::{usart, usart::Uart};
use embassy_sync::signal::Signal;

use heapless::box_pool;
use heapless::pool::boxed::{Box, BoxBlock};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use futures::FutureExt;

use tp_led_matrix::{matrix::Matrix, Color, Image};

static NEXT_IMAGE: Signal<ThreadModeRawMutex, Box<POOL>> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("defmt correctly initialized");

    // Setup the clocks at 80MHz using HSI (by default since HSE/MSI
    // are not configured): HSI(16MHz)×10/2=80MHz. The flash wait
    // states will be configured accordingly.
    let mut config = Config::default();
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2), // 16 * 10 / 2 = 80MHz
    });
    let p = embassy_stm32::init(config);

    let matrix = Matrix::new(
        p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,
    )
    .await;

    //==============================================================================//

    #[allow(clippy::declare_interior_mutable_const)]
    unsafe {
        const BLOCK: BoxBlock<Image> = BoxBlock::new();
        static mut MEMORY: [BoxBlock<Image>; 3] = [BLOCK; 3];
        #[allow(static_mut_refs)]
        for block in &mut MEMORY {
            POOL.manage(block);
        }
    }

    if let Ok(image) = POOL.alloc(Image::gradient(Color::RED)) {
        NEXT_IMAGE.signal(image);
    }

    let _ = spawner.spawn(blinker(p.PB14));
    let _ = spawner.spawn(serial_receiver(p.USART1, p.PB6, p.PB7, p.DMA1_CH5));
    spawner.spawn(display_task(matrix)).unwrap(); // Spawn the display task with the image
}

//==============================================================================//
#[embassy_executor::task]
/// Task that blinks the LED on the board, receiving the led pin as an argument.
async fn blinker(led: PB14) {
    let mut led = Output::new(led, Level::Low, Speed::VeryHigh);

    loop {
        for _ in 0..3 {
            led.set_high();
            Timer::after_millis(100).await;
            led.set_low();
            Timer::after_millis(100).await;
        }

        Timer::after_millis(1000).await;
    }
}
//==============================================================================//
#[embassy_executor::task]
async fn display_task(mut matrix: Matrix<'static>) {
    let mut ticker = Ticker::every(Duration::from_hz(640)); // Setup ticker for frame timing
    let mut current_image = NEXT_IMAGE.wait().await; // Wait for the first image from serial_receiver

    loop {
        // Check if a new image is available and replace the current one if so
        let generated_image = NEXT_IMAGE.wait().now_or_never();
        if generated_image.is_some() {
            current_image = generated_image.unwrap();
        }
        matrix.display_image(&current_image, &mut ticker).await;
        ticker.next().await;
    }
}

//==============================================================================//
bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
});

box_pool!(POOL: Image);
//==============================================================================//
#[embassy_executor::task]
async fn serial_receiver(usart1: USART1, pb6: PB6, pb7: PB7, dma1_ch5: DMA1_CH5) {
    defmt::info!("Serial receiver task started");

    let mut config = usart::Config::default();
    config.baudrate = 38400;
    let mut uart = Uart::new(usart1, pb7, pb6, Irqs, NoDma, dma1_ch5, config).unwrap();

    loop {
        let mut byte: u8 = 0u8;
        uart.read(core::slice::from_mut(&mut byte)).await.unwrap();
        if byte != 0xFF {
            continue;
        }

        let Ok(mut image) = POOL.alloc(Image::default()) else {
            continue;
        };

        let mut start = 0;
        'receive: loop {
            uart.read(&mut image.as_mut()[start..]).await.unwrap();
            for pos in (start..192).rev() {
                if image.as_ref()[pos] == 0xFF {
                    image.as_mut().rotate_left(pos + 1);
                    start = 192 - (pos + 1);
                    continue 'receive;
                }
            }
            break;
        }
        NEXT_IMAGE.signal(image);
    }
}

//==============================================================================//
