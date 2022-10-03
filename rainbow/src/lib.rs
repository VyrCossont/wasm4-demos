#![no_std]

mod wasm4;

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

const CHAR_W: usize = 8;
const CHAR_H: usize = 8;
const MSG: &str = "wasm-4";
const MSG_LEN: usize = MSG.len();
const MSG_PIXELS_PER_BYTE: usize = 8;
const SCALE: usize = 3;
const MSG_W: usize = CHAR_W * SCALE * MSG_LEN;
const MSG_H: usize = CHAR_H * SCALE;
const MSG_NUM_PIXELS: usize = MSG_W * MSG_H;
const MSG_NUM_BYTES: usize = MSG_NUM_PIXELS / MSG_PIXELS_PER_BYTE
    + if MSG_NUM_PIXELS % MSG_PIXELS_PER_BYTE > 0 {
        1
    } else {
        0
    };

struct State {
    clock: i32,
    msg_scaled: [u8; MSG_NUM_BYTES],
}

static mut STATE: State = State {
    clock: 0,
    msg_scaled: [0; MSG_NUM_BYTES],
};

const FRAMEBUFFER_PIXELS_PER_BYTE: usize = 4;
const FRAMEBUFFER_BITS_PER_PIXEL: usize = u8::BITS as usize / FRAMEBUFFER_PIXELS_PER_BYTE;
const FRAMEBUFFER_PIXEL_MASK: u8 = ((1 << FRAMEBUFFER_BITS_PER_PIXEL) - 1) as u8;

fn get_framebuffer_pixel(x: usize, y: usize) -> bool {
    let framebuffer = unsafe { &*wasm4::FRAMEBUFFER };
    let pixel_offset = y * wasm4::SCREEN_SIZE as usize + x;
    let byte_offset = pixel_offset / FRAMEBUFFER_PIXELS_PER_BYTE;
    let shift = (pixel_offset % FRAMEBUFFER_PIXELS_PER_BYTE) * FRAMEBUFFER_BITS_PER_PIXEL;
    let byte = framebuffer[byte_offset];
    (byte >> shift) & FRAMEBUFFER_PIXEL_MASK != 0
}

const MSG_BITS_PER_PIXEL: usize = u8::BITS as usize / MSG_PIXELS_PER_BYTE;
const MSG_PIXEL_MASK: u8 = ((1 << MSG_BITS_PER_PIXEL) - 1) as u8;

fn set_msg_scaled_pixel(state: &mut State, x: usize, y: usize, pixel: bool) {
    let pixel_offset = y * MSG_W + x;
    let byte_offset = pixel_offset / MSG_PIXELS_PER_BYTE;
    let shift = (u8::BITS as usize - 1) - (pixel_offset % MSG_PIXELS_PER_BYTE) * MSG_BITS_PER_PIXEL;
    let mut byte = state.msg_scaled[byte_offset];
    byte &= !(MSG_PIXEL_MASK << shift);
    if pixel {
        byte |= MSG_PIXEL_MASK << shift;
    }
    state.msg_scaled[byte_offset] = byte;
}

const SCREEN_MAX: usize = wasm4::SCREEN_SIZE as usize - 1;

#[no_mangle]
fn start() {
    let state = unsafe { &mut STATE };
    let palette = unsafe { &mut *wasm4::PALETTE };
    let draw_colors = unsafe { &mut *wasm4::DRAW_COLORS };

    // This color is for background and text and won't change.
    palette[0] = 0x00_00_00;

    // Draw the message to the screen using any color other than background.
    *draw_colors = 2;
    wasm4::text(MSG, 0, 0);

    // Copy and scale it into the scaled message buffer using scale3x:
    // https://en.wikipedia.org/wiki/Pixel-art_scaling_algorithms#Scale3%C3%97/AdvMAME3%C3%97_and_ScaleFX
    for y in 0..CHAR_H {
        for x in 0..CHAR_W * MSG_LEN {
            let a = x > 0 && y > 0 && get_framebuffer_pixel(x - 1, y - 1);
            let b = y > 0 && get_framebuffer_pixel(x, y - 1);
            let c = x < SCREEN_MAX && y > 0 && get_framebuffer_pixel(x + 1, y - 1);
            let d = x > 0 && get_framebuffer_pixel(x - 1, y);
            let e = get_framebuffer_pixel(x, y);
            let f = x < SCREEN_MAX && get_framebuffer_pixel(x + 1, y);
            let g = x > 0 && y < SCREEN_MAX && get_framebuffer_pixel(x - 1, y + 1);
            let h = y > 0 && get_framebuffer_pixel(x, y + 1);
            let i = x < SCREEN_MAX && y < SCREEN_MAX && get_framebuffer_pixel(x + 1, y + 1);

            let p1 = if d == b && d != h && b != f { d } else { e };
            let p2 = if (d == b && d != h && b != f && e != c)
                || (b == f && b != d && f != h && e != a)
            {
                b
            } else {
                e
            };
            let p3 = if b == f && b != d && f != h { f } else { e };
            let p4 = if (h == d && h != f && d != b && e != a)
                || (d == b && d != h && b != f && e != g)
            {
                d
            } else {
                e
            };
            let p5 = e;
            let p6 = if (b == f && b != d && f != h && e != i)
                || (f == h && f != b && h != d && e != c)
            {
                f
            } else {
                e
            };
            let p7 = if h == d && h != f && d != b { d } else { e };
            let p8 = if (f == h && f != b && h != d && e != g)
                || (h == d && h != f && d != b && e != i)
            {
                h
            } else {
                e
            };
            let p9 = if f == h && f != b && h != d { f } else { e };

            set_msg_scaled_pixel(state, x * 3, y * 3, p1);
            set_msg_scaled_pixel(state, x * 3 + 1, y * 3, p2);
            set_msg_scaled_pixel(state, x * 3 + 2, y * 3, p3);
            set_msg_scaled_pixel(state, x * 3, y * 3 + 1, p4);
            set_msg_scaled_pixel(state, x * 3 + 1, y * 3 + 1, p5);
            set_msg_scaled_pixel(state, x * 3 + 2, y * 3 + 1, p6);
            set_msg_scaled_pixel(state, x * 3, y * 3 + 2, p7);
            set_msg_scaled_pixel(state, x * 3 + 1, y * 3 + 2, p8);
            set_msg_scaled_pixel(state, x * 3 + 2, y * 3 + 2, p9);
        }
    }
}

const SCREEN_COLORS: [u32; 7] = [
    0xff_00_00, 0xff_7f_00, 0xff_ff_00, 0x00_ff_00, 0x00_00_ff, 0x4b_00_82, 0x94_00_d3,
];

const STRIPE_HEIGHT: i32 = wasm4::SCREEN_SIZE as i32 / SCREEN_COLORS.len() as i32
    + if wasm4::SCREEN_SIZE as i32 % SCREEN_COLORS.len() as i32 != 0 {
        1
    } else {
        0
    };
const NUM_CYCLED_COLORS: i32 = 3;
const WINDOW_HEIGHT: i32 = (NUM_CYCLED_COLORS - 1) * STRIPE_HEIGHT;
const SPEED: i32 = 4;

const SCREEN_CENTER: i32 = wasm4::SCREEN_SIZE as i32 / 2;
const MSG_X: i32 = SCREEN_CENTER - MSG_W as i32 / 2;
const MSG_Y: i32 = SCREEN_CENTER - MSG_H as i32 / 2;

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

    // Draw text.
    *draw_colors = 0x10;
    wasm4::blit(
        &state.msg_scaled,
        MSG_X,
        MSG_Y,
        MSG_W as u32,
        MSG_H as u32,
        wasm4::BLIT_1BPP,
    );

    state.clock += SPEED;
    if state.clock >= wasm4::SCREEN_SIZE as i32 {
        state.clock = -WINDOW_HEIGHT + 1;
    }
}
