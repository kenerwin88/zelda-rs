use crate::bg_layer::BgLayerRenderer;
use crate::gpu_frame::GpuFrame;
use crate::gpu_frame_renderer_backend::GpuFrameRendererBackend;
use crate::mode7_renderer::Mode7Renderer;
use crate::post_process::PostProcessRenderer;
use crate::sprite_renderer::SpriteRenderer;
use crate::tile_atlas::{CgramPalette, RgbaTileOverrideData, RgbaTileOverrideTextures, TileAtlas};

pub(crate) struct GpuFrameRendererResources {
    tile_atlas: TileAtlas,
    cgram_palette: CgramPalette,
    _rgba_tile_overrides: RgbaTileOverrideTextures,
    /// bg[0]=BG1, bg[1]=BG2, bg[2]=BG3; indices match `GpuFrame.bg[]`.
    bg: [BgLayerRenderer; 3],
    mode7: Mode7Renderer,
    sprites: SpriteRenderer,
    post_process: PostProcessRenderer,
    /// Intermediate texture: main-screen BG+sprite composite before color math.
    #[allow(dead_code)]
    comp_tex: wgpu::Texture,
    comp_view: wgpu::TextureView,
    /// Intermediate texture: sub-screen composite (second operand for color math).
    #[allow(dead_code)]
    sub_comp_tex: wgpu::Texture,
    sub_comp_view: wgpu::TextureView,
}

const COMP_WIDTH: u32 = 256;
const COMP_HEIGHT: u32 = 224;

impl GpuFrameRendererResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba_tile_overrides: Option<RgbaTileOverrideData<'_>>,
    ) -> Self {
        let tile_atlas = TileAtlas::new(device);
        let cgram_palette = CgramPalette::new(device);
        let rgba_tile_overrides = RgbaTileOverrideTextures::new(device, queue, rgba_tile_overrides);
        let bg = [
            BgLayerRenderer::new(
                device,
                &tile_atlas,
                &cgram_palette,
                &rgba_tile_overrides,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
            BgLayerRenderer::new(
                device,
                &tile_atlas,
                &cgram_palette,
                &rgba_tile_overrides,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
            BgLayerRenderer::new(
                device,
                &tile_atlas,
                &cgram_palette,
                &rgba_tile_overrides,
                wgpu::TextureFormat::Rgba8Unorm,
            ),
        ];
        let sprites = SpriteRenderer::new(
            device,
            &tile_atlas,
            &cgram_palette,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let mode7 = Mode7Renderer::new(device, &cgram_palette, wgpu::TextureFormat::Rgba8Unorm);
        let comp_tex_desc = wgpu::TextureDescriptor {
            label: Some("comp_intermediate"),
            size: wgpu::Extent3d {
                width: COMP_WIDTH,
                height: COMP_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let comp_tex = device.create_texture(&comp_tex_desc);
        let comp_view = comp_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sub_comp_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sub_comp_intermediate"),
            ..comp_tex_desc
        });
        let sub_comp_view = sub_comp_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let post_process = PostProcessRenderer::new(
            device,
            wgpu::TextureFormat::Rgba8Unorm,
            &comp_view,
            &sub_comp_view,
        );
        Self {
            tile_atlas,
            cgram_palette,
            _rgba_tile_overrides: rgba_tile_overrides,
            bg,
            mode7,
            sprites,
            post_process,
            comp_tex,
            comp_view,
            sub_comp_tex,
            sub_comp_view,
        }
    }

    pub(crate) fn backend<'a, 'frame>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
        queue: &'a wgpu::Queue,
        frame: &'a GpuFrame<'frame>,
        output_view: &'a wgpu::TextureView,
        mode7_source_chars: Option<&'a [u8]>,
    ) -> GpuFrameRendererBackend<'a, 'frame> {
        GpuFrameRendererBackend {
            tile_atlas: &mut self.tile_atlas,
            cgram_palette: &mut self.cgram_palette,
            bg: &mut self.bg,
            mode7: &mut self.mode7,
            sprites: &mut self.sprites,
            post_process: &mut self.post_process,
            comp_view: &self.comp_view,
            sub_comp_view: &self.sub_comp_view,
            encoder,
            queue,
            frame,
            output_view,
            mode7_source_chars,
        }
    }
}
