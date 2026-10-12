use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, Stack, StackResources};
use embassy_net_wiznet::{Device, State, chip::W5500};
use embassy_rp::{
    Peri, bind_interrupts, dma,
    gpio::{Input, Level, Output, Pull},
    peripherals::{DMA_CH0, DMA_CH1, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, SPI0},
    spi::{Async, Config, Spi},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;

type SpiDevice = ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, embassy_time::Delay>;
type WiznetRunner =
    embassy_net_wiznet::Runner<'static, W5500, SpiDevice, Input<'static>, Output<'static>>;

pub(crate) struct Pins {
    pub(crate) spi: Peri<'static, SPI0>,
    pub(crate) miso: Peri<'static, PIN_16>,
    pub(crate) cs: Peri<'static, PIN_17>,
    pub(crate) clk: Peri<'static, PIN_18>,
    pub(crate) mosi: Peri<'static, PIN_19>,
    pub(crate) reset: Peri<'static, PIN_20>,
    pub(crate) int: Peri<'static, PIN_21>,
    pub(crate) tx_dma: Peri<'static, DMA_CH0>,
    pub(crate) rx_dma: Peri<'static, DMA_CH1>,
}

bind_interrupts!(struct Irqs{
    DMA_IRQ_0 =>
        dma::InterruptHandler<DMA_CH0>,
        dma::InterruptHandler<DMA_CH1>;
});

static STATE: StaticCell<State<4, 4>> = StaticCell::new();
static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
const MAC: [u8; 6] = [0x02, 0x45, 0x52, 0x49, 0x4b, 0x01];

#[embassy_executor::task]
async fn w5500_task(runner: WiznetRunner) {
    runner.run().await;
}

#[embassy_executor::task]
async fn stack_task(mut runner: embassy_net::Runner<'static, Device<'static>>) {
    runner.run().await;
}

pub(crate) async fn start(spawner: Spawner, pins: Pins) -> Stack<'static> {
    let mut spi_config = Config::default();
    spi_config.frequency = 10_000_000;

    let mut dhcp_config = DhcpConfig::default();
    dhcp_config.hostname = Some("w5500-pico-playground".try_into().unwrap());

    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let spi = Spi::new(
        pins.spi,
        pins.clk,
        pins.mosi,
        pins.miso,
        pins.tx_dma,
        pins.rx_dma,
        Irqs,
        spi_config,
    );

    let cs = Output::new(pins.cs, Level::High);
    let reset = Output::new(pins.reset, Level::High);
    let int = Input::new(pins.int, Pull::Up);

    let spi_device = ExclusiveDevice::new(spi, cs, embassy_time::Delay).unwrap();

    let (device, w5500_runner) = embassy_net_wiznet::new::<4, 4, W5500, _, _, _>(
        MAC,
        STATE.init_with(State::new),
        spi_device,
        int,
        reset,
    )
    .await
    .unwrap();

    spawner.spawn(w5500_task(w5500_runner).unwrap());

    let seed = embassy_rp::clocks::RoscRng.next_u64();

    let (stack, stack_runner) = embassy_net::new(
        device,
        net_config,
        RESOURCES.init_with(StackResources::new),
        seed,
    );

    spawner.spawn(stack_task(stack_runner).unwrap());

    stack
}
