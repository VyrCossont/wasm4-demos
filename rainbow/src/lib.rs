#![no_std]

mod wasm4;

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

struct State {
    clock: i32,
}

static mut STATE: State = State { clock: 0 };

#[no_mangle]
fn start() {
    let palette = unsafe { &mut *wasm4::PALETTE };
    palette[0] = 0x00_00_00;
}

const SCREEN_COLORS: [u32; 7] = [
    0xff_00_00, 0xff_7f_00, 0xff_ff_00, 0x00_ff_00, 0x00_00_ff, 0x4b_00_82, 0x94_00_d3,
];

const STRIPE_HEIGHT: i32 = 8;
const NUM_CYCLED_COLORS: i32 = 3;
const WINDOW_HEIGHT: i32 = (NUM_CYCLED_COLORS - 1) * STRIPE_HEIGHT;
const SPEED: i32 = 4;

#[no_mangle]
fn update() {
    let state = unsafe { &mut STATE };
    let palette = unsafe { &mut *wasm4::PALETTE };
    let draw_colors = unsafe { &mut *wasm4::DRAW_COLORS };

    // Set up palette.
    {
        let stripe = state.clock / STRIPE_HEIGHT;
        let draw_color = stripe.rem_euclid(NUM_CYCLED_COLORS);
        let screen_color = stripe.rem_euclid(SCREEN_COLORS.len() as i32);
        for i in 0..NUM_CYCLED_COLORS {
            palette[(1 + ((draw_color + i) % NUM_CYCLED_COLORS)) as usize] =
                SCREEN_COLORS[(screen_color + i) as usize % SCREEN_COLORS.len()];
        }
    }

    // Draw colored stripes.
    for y in state.clock..state.clock + WINDOW_HEIGHT {
        let draw_color = (2 + (y / STRIPE_HEIGHT) % NUM_CYCLED_COLORS) as u16;
        *draw_colors = draw_color;
        wasm4::hline(0, y, wasm4::SCREEN_SIZE);
    }

    state.clock += SPEED;
    if state.clock >= wasm4::SCREEN_SIZE as i32 {
        state.clock = -WINDOW_HEIGHT + 1;
    }
}
