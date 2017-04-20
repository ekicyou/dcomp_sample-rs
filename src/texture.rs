use super::com::*;
use super::consts::*;

// Generate a simple black and white checkerboard texture.
pub fn generate_texture_data()->Vec<u8>
{
	let  row_pitch:u32 = TEXTURE_WIDTH * sizeof(u32);
	let  cell_pitch:u32 = row_pitch >> 3;		// The width of a cell in the checkboard texture.
	let  cell_height:u32 = TEXTURE_WIDTH >> 3;	// The height of a cell in the checkerboard texture.
	let  texture_size:u32 = row_pitch * TEXTURE_HEIGHT;

    DirectX::XMFLOAT4 colors[NUM_TEXTURE_COLORS] =
    {
        DirectX::XMFLOAT4(1, 0, 0, 1), // Red
        DirectX::XMFLOAT4(0, 1, 0, 1), // Green
        DirectX::XMFLOAT4(0, 0, 1, 1), // Blue
        DirectX::XMFLOAT4(0, 0, 0, 1), // Black
        DirectX::XMFLOAT4(1, 1, 1, 1), // White
        DirectX::XMFLOAT4(1, 1, 0, 1), // Yellow
        DirectX::XMFLOAT4(0, 1, 1, 1), // Cyan
        DirectX::XMFLOAT4(1, 0, 1, 1)  // Purple
    };

	std::vector<u8> data(texture_size);
    u8* pData = &data[0];

    for (u32 a = 0; a < NUM_ALPHA_SHADES; ++a)
    {
        float alpha = a / (float)(NUM_ALPHA_SHADES - 1);
        u32 start_x = a * TEXTURE_PIXEL_SIZE_X;
        u32 end_x = start_x + TEXTURE_PIXEL_SIZE_X;

        for (u32 c = 0; c < NUM_TEXTURE_COLORS; ++c)
        {
            const DirectX::XMFLOAT4& color = colors[c];
            DirectX::XMFLOAT4 pmaColor = 
            { 
                color.x * alpha,
                color.y * alpha,
                color.z * alpha,
                alpha
            };

            u32 start_y = TEXTURE_PIXEL_SIZE_Y * c;
            u32 end_y = start_y + TEXTURE_PIXEL_SIZE_Y;
            for (u32 y = start_y; y < end_y; ++y)
            {
                for (u32 x = start_x; x < end_x; ++x)
                {
                    u32 offset = (y * TEXTURE_WIDTH + x) * sizeof(u32);
                    pData[offset + 0] = (uint8_t)(pmaColor.x * 255.0f);
                    pData[offset + 1] = (uint8_t)(pmaColor.y * 255.0f);
                    pData[offset + 2] = (uint8_t)(pmaColor.z * 255.0f);
                    pData[offset + 3] = (uint8_t)(pmaColor.w * 255.0f);
                }
            }
        }
	}

	return data;
}
