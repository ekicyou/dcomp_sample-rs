
// Generate a simple black and white checkerboard texture.
pub fn generate_texture_data()->Vec<u8>
{
	let  rowPitch:u32 = TextureWidth * sizeof(UINT);
	let  cellPitch:u32 = rowPitch >> 3;		// The width of a cell in the checkboard texture.
	let  cellHeight:u32 = TextureWidth >> 3;	// The height of a cell in the checkerboard texture.
	let  textureSize:u32 = rowPitch * TextureHeight;

    DirectX::XMFLOAT4 colors[NumTextureColors] =
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

	std::vector<UINT8> data(textureSize);
    UINT8* pData = &data[0];

    for (UINT a = 0; a < NumAlphaShades; ++a)
    {
        float alpha = a / (float)(NumAlphaShades - 1);
        UINT start_x = a * TexturePixelSizeX;
        UINT end_x = start_x + TexturePixelSizeX;

        for (UINT c = 0; c < NumTextureColors; ++c)
        {
            const DirectX::XMFLOAT4& color = colors[c];
            DirectX::XMFLOAT4 pmaColor = 
            { 
                color.x * alpha,
                color.y * alpha,
                color.z * alpha,
                alpha
            };

            UINT start_y = TexturePixelSizeY * c;
            UINT end_y = start_y + TexturePixelSizeY;
            for (UINT y = start_y; y < end_y; ++y)
            {
                for (UINT x = start_x; x < end_x; ++x)
                {
                    UINT offset = (y * TextureWidth + x) * sizeof(UINT);
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
