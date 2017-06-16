use super::com::*;
use super::consts::*;
use winapi::_core::mem;

// Generate a simple black and white checkerboard texture.
pub fn generate_texture_data() -> Vec<u8> {
    let row_pitch = mem::size_of::<u32>() * TEXTURE_WIDTH as usize;
    let cell_pitch = (row_pitch >> 3) as usize; // The width of a cell in the checkboard texture.
    let cell_height = (TEXTURE_WIDTH >> 3) as usize; // The height of a cell in the checkerboard texture.
    let texture_size = row_pitch * TEXTURE_HEIGHT as usize;

    let colors = [
        XMFLOAT4::new(1_f32, 0_f32, 0_f32, 1_f32), // Red
        XMFLOAT4::new(0_f32, 1_f32, 0_f32, 1_f32), // Green
        XMFLOAT4::new(0_f32, 0_f32, 1_f32, 1_f32), // Blue
        XMFLOAT4::new(0_f32, 0_f32, 0_f32, 1_f32), // Black
        XMFLOAT4::new(1_f32, 1_f32, 1_f32, 1_f32), // White
        XMFLOAT4::new(1_f32, 1_f32, 0_f32, 1_f32), // Yellow
        XMFLOAT4::new(0_f32, 1_f32, 1_f32, 1_f32), // Cyan
        XMFLOAT4::new(1_f32, 0_f32, 1_f32, 1_f32)  // Purple
    ];

    let mut buf: Vec<u8> = Vec::new();
    buf.resize(texture_size, 0_u8);
    let mut data = buf.as_mut_slice();

    for a in 0_usize..NUM_ALPHA_SHADES as _ {
        let alpha = (a as f32) / ((NUM_ALPHA_SHADES - 1) as f32);
        let start_x = a * TEXTURE_PIXEL_SIZE_X as usize;
        let end_x = start_x + TEXTURE_PIXEL_SIZE_X as usize;

        for c in 0_usize..NUM_TEXTURE_COLORS as _ {
            let color = colors[c];
            let pma_color = XMFLOAT4 {
                x: color.x * alpha,
                y: color.y * alpha,
                z: color.z * alpha,
                w: alpha,
            };

            let start_y = c * TEXTURE_PIXEL_SIZE_Y as usize;
            let end_y = start_y + TEXTURE_PIXEL_SIZE_Y as usize;
            for y in start_y..end_y {
                for x in start_x..end_x {
                    let offset = (y * TEXTURE_WIDTH as usize + x) *
                        mem::size_of::<u32>() as usize;
                    data[offset + 0] = (pma_color.x * 255_f32) as u8;
                    data[offset + 1] = (pma_color.y * 255_f32) as u8;
                    data[offset + 2] = (pma_color.z * 255_f32) as u8;
                    data[offset + 3] = (pma_color.w * 255_f32) as u8;
                }
            }
        }
    }
    buf
}
