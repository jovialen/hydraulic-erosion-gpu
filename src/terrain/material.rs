use super::Terrain;
use bevy::asset::Handle;
use bevy::math::Vec4;
use bevy::pbr::{
    AlphaMode, Material, MaterialPipeline, MaterialPipelineKey, ParallaxMappingMethod,
    PBR_PREPASS_SHADER_HANDLE, PBR_SHADER_HANDLE,
};
use bevy::reflect::{std_traits::ReflectDefault, Reflect, TypePath, TypeUuid};
use bevy::render::{
    color::Color, mesh::MeshVertexBufferLayout, render_asset::RenderAssets, render_resource::*,
    texture::Image,
};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};

impl From<Terrain> for Image {
    fn from(terrain: Terrain) -> Self {
        let usize_size = terrain.size as usize;
        let f64_scale = terrain.scale as f64;

        let fbm: Fbm<Perlin> = Fbm::default();

        let data = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(fbm)
            .set_size(usize_size, usize_size)
            .set_x_bounds(0.0, f64_scale)
            .set_y_bounds(0.0, f64_scale)
            .build()
            .iter()
            .map(|v| (v * 255.0).floor() as u8)
            .collect();

        Image::new(
            Extent3d {
                width: terrain.size,
                height: terrain.size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::R8Unorm,
        )
    }
}

#[derive(AsBindGroup, Debug, Clone, TypePath, TypeUuid)]
#[uuid = "0e1d21c6-dd0b-4780-a774-9d6153786f61"]
#[bind_group_data(StandardMaterialKey)]
#[uniform(0, StandardMaterialUniform)]
pub struct TerrainMaterial {
    pub base_color: Color,

    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,

    pub emissive: Color,

    #[texture(3)]
    #[sampler(4)]
    pub emissive_texture: Option<Handle<Image>>,

    pub perceptual_roughness: f32,
    pub metallic: f32,

    #[texture(5)]
    #[sampler(6)]
    pub metallic_roughness_texture: Option<Handle<Image>>,

    #[doc(alias = "specular_intensity")]
    pub reflectance: f32,

    #[texture(9)]
    #[sampler(10)]
    pub normal_map_texture: Option<Handle<Image>>,

    pub flip_normal_map_y: bool,

    #[texture(7)]
    #[sampler(8)]
    pub occlusion_texture: Option<Handle<Image>>,

    pub double_sided: bool,
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub fog_enabled: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,

    #[texture(11)]
    #[sampler(12)]
    pub depth_map: Option<Handle<Image>>,

    pub parallax_depth_scale: f32,
    pub parallax_mapping_method: ParallaxMappingMethod,
    pub max_parallax_layer_count: f32,

    #[texture(13)]
    #[sampler(14)]
    pub heightmap: Handle<Image>,
}

impl From<Handle<Image>> for TerrainMaterial {
    fn from(heightmap: Handle<Image>) -> Self {
        TerrainMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            base_color_texture: None,
            emissive: Color::BLACK,
            emissive_texture: None,
            perceptual_roughness: 0.5,
            metallic: 0.0,
            metallic_roughness_texture: None,
            reflectance: 0.5,
            occlusion_texture: None,
            normal_map_texture: None,
            flip_normal_map_y: false,
            double_sided: false,
            cull_mode: Some(Face::Back),
            unlit: false,
            fog_enabled: true,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            depth_map: None,
            parallax_depth_scale: 0.1,
            max_parallax_layer_count: 16.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
            heightmap,
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct StandardMaterialFlags: u32 {
        const BASE_COLOR_TEXTURE         = (1 << 0);
        const EMISSIVE_TEXTURE           = (1 << 1);
        const METALLIC_ROUGHNESS_TEXTURE = (1 << 2);
        const OCCLUSION_TEXTURE          = (1 << 3);
        const DOUBLE_SIDED               = (1 << 4);
        const UNLIT                      = (1 << 5);
        const TWO_COMPONENT_NORMAL_MAP   = (1 << 6);
        const FLIP_NORMAL_MAP_Y          = (1 << 7);
        const FOG_ENABLED                = (1 << 8);
        const DEPTH_MAP                  = (1 << 9);
        const ALPHA_MODE_RESERVED_BITS   = (Self::ALPHA_MODE_MASK_BITS << Self::ALPHA_MODE_SHIFT_BITS);
        const ALPHA_MODE_OPAQUE          = (0 << Self::ALPHA_MODE_SHIFT_BITS);
        const ALPHA_MODE_MASK            = (1 << Self::ALPHA_MODE_SHIFT_BITS);
        const ALPHA_MODE_BLEND           = (2 << Self::ALPHA_MODE_SHIFT_BITS);
        const ALPHA_MODE_PREMULTIPLIED   = (3 << Self::ALPHA_MODE_SHIFT_BITS);
        const ALPHA_MODE_ADD             = (4 << Self::ALPHA_MODE_SHIFT_BITS);
        const ALPHA_MODE_MULTIPLY        = (5 << Self::ALPHA_MODE_SHIFT_BITS);
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

impl StandardMaterialFlags {
    const ALPHA_MODE_MASK_BITS: u32 = 0b111;
    const ALPHA_MODE_SHIFT_BITS: u32 = 32 - Self::ALPHA_MODE_MASK_BITS.count_ones();
}

#[derive(Clone, Default, ShaderType)]
pub struct StandardMaterialUniform {
    pub base_color: Vec4,
    pub emissive: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub flags: u32,
    pub alpha_cutoff: f32,
    pub parallax_depth_scale: f32,
    pub max_parallax_layer_count: f32,
    pub max_relief_mapping_search_steps: u32,
}

impl AsBindGroupShaderType<StandardMaterialUniform> for TerrainMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> StandardMaterialUniform {
        let mut flags = StandardMaterialFlags::NONE;
        if self.base_color_texture.is_some() {
            flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        }
        if self.emissive_texture.is_some() {
            flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        }
        if self.metallic_roughness_texture.is_some() {
            flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }
        if self.occlusion_texture.is_some() {
            flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        }
        if self.double_sided {
            flags |= StandardMaterialFlags::DOUBLE_SIDED;
        }
        if self.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }
        if self.fog_enabled {
            flags |= StandardMaterialFlags::FOG_ENABLED;
        }
        if self.depth_map.is_some() {
            flags |= StandardMaterialFlags::DEPTH_MAP;
        }
        let has_normal_map = self.normal_map_texture.is_some();
        if has_normal_map {
            if let Some(texture) = images.get(self.normal_map_texture.as_ref().unwrap()) {
                match texture.texture_format {
                    TextureFormat::Rg8Unorm
                    | TextureFormat::Rg16Unorm
                    | TextureFormat::Bc5RgUnorm
                    | TextureFormat::EacRg11Unorm => {
                        flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
                    }
                    _ => {}
                }
            }
            if self.flip_normal_map_y {
                flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
            }
        }
        let mut alpha_cutoff = 0.5;
        match self.alpha_mode {
            AlphaMode::Opaque => flags |= StandardMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= StandardMaterialFlags::ALPHA_MODE_BLEND,
            AlphaMode::Premultiplied => flags |= StandardMaterialFlags::ALPHA_MODE_PREMULTIPLIED,
            AlphaMode::Add => flags |= StandardMaterialFlags::ALPHA_MODE_ADD,
            AlphaMode::Multiply => flags |= StandardMaterialFlags::ALPHA_MODE_MULTIPLY,
        };

        StandardMaterialUniform {
            base_color: self.base_color.as_linear_rgba_f32().into(),
            emissive: self.emissive.as_linear_rgba_f32().into(),
            roughness: self.perceptual_roughness,
            metallic: self.metallic,
            reflectance: self.reflectance,
            flags: flags.bits(),
            alpha_cutoff,
            parallax_depth_scale: self.parallax_depth_scale,
            max_parallax_layer_count: self.max_parallax_layer_count,
            max_relief_mapping_search_steps: match self.parallax_mapping_method {
                ParallaxMappingMethod::Occlusion => 0,
                ParallaxMappingMethod::Relief { max_steps } => max_steps,
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StandardMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
    depth_bias: i32,
    relief_mapping: bool,
}

impl From<&TerrainMaterial> for StandardMaterialKey {
    fn from(material: &TerrainMaterial) -> Self {
        StandardMaterialKey {
            normal_map: material.normal_map_texture.is_some(),
            cull_mode: material.cull_mode,
            depth_bias: material.depth_bias as i32,
            relief_mapping: matches!(
                material.parallax_mapping_method,
                ParallaxMappingMethod::Relief { .. }
            ),
        }
    }
}

impl Material for TerrainMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            let shader_defs = &mut fragment.shader_defs;

            if key.bind_group_data.normal_map {
                shader_defs.push("STANDARDMATERIAL_NORMAL_MAP".into());
            }
            if key.bind_group_data.relief_mapping {
                shader_defs.push("RELIEF_MAPPING".into());
            }
        }
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        if let Some(label) = &mut descriptor.label {
            *label = format!("pbr_{}", *label).into();
        }
        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            depth_stencil.bias.constant = key.bind_group_data.depth_bias;
        }
        Ok(())
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        PBR_PREPASS_SHADER_HANDLE.typed().into()
    }

    fn fragment_shader() -> ShaderRef {
        PBR_SHADER_HANDLE.typed().into()
    }

    #[inline]
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    #[inline]
    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
}
