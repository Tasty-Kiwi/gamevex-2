use bevy::{
    asset::{RenderAssetUsages, load_internal_asset, uuid_handle},
    camera::{ScalingMode, visibility::RenderLayers},
    image::ImageSampler,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, Extent3d, TextureFormat, TextureUsages},
    shader::ShaderRef,
    sprite::Anchor,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

const CRT_SHADER: Handle<Shader> = uuid_handle!("e1e7c94d-f608-4672-af98-4412e77052d1");

const SCENE_LAYER: usize = 0;
const TEXT_LAYER: usize = 1;
const COMPOSITE_LAYER: usize = 2;
const LOGICAL_PIXEL_SCALE: f32 = 0.5;

/// Immediate-mode, top-left-origin drawing commands for one logical frame.
#[derive(Resource, Default)]
pub struct Frame {
    pub size: Vec2,
    pub min_stroke: f32,
    pub palette: gamevex_core::ColorScheme,
    vertices: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    text: Vec<TextCommand>,
}

#[derive(Clone)]
struct TextCommand {
    pos: Vec2,
    text: String,
    font_size: f32,
    color: Color,
}

impl Frame {
    /// Clear this frame's draw list and set its logical pixel dimensions.
    pub fn clear(&mut self, size: Vec2) {
        self.size = size.max(Vec2::ZERO);
        self.vertices.clear();
        self.colors.clear();
        self.indices.clear();
        self.text.clear();
    }

    /// Add a solid rectangle with a top-left position and positive dimensions.
    pub fn rect(&mut self, pos: Vec2, size: Vec2, color: Color) {
        if !valid_size(self.size) || !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
            return;
        }
        self.push_axis_quad(pos, pos + size, color);
    }

    /// Add a thick line as a solid quadrilateral.
    pub fn line(&mut self, a: Vec2, b: Vec2, width: f32, color: Color) {
        let width = width.max(self.min_stroke);
        let delta = b - a;
        let length = delta.length();
        if !valid_size(self.size)
            || !a.is_finite()
            || !b.is_finite()
            || !length.is_finite()
            || !width.is_finite()
            || length <= f32::EPSILON
            || width <= 0.0
        {
            return;
        }
        let normal = Vec2::new(-delta.y, delta.x) / length * (width * 0.5);
        self.push_quad(a + normal, b + normal, b - normal, a - normal, color);
    }

    /// Add an outlined rectangle using four thick lines.
    pub fn outline(&mut self, pos: Vec2, size: Vec2, width: f32, color: Color) {
        if !pos.is_finite()
            || !size.is_finite()
            || !width.is_finite()
            || size.x <= 0.0
            || size.y <= 0.0
            || width <= 0.0
        {
            return;
        }
        let top_left = pos;
        let top_right = pos + Vec2::new(size.x, 0.0);
        let bottom_right = pos + size;
        let bottom_left = pos + Vec2::new(0.0, size.y);
        self.line(top_left, top_right, width, color);
        self.line(top_right, bottom_right, width, color);
        self.line(bottom_right, bottom_left, width, color);
        self.line(bottom_left, top_left, width, color);
    }

    /// Add a rounded rectangular outline using batched thick line segments.
    /// The radius is clamped to half the rectangle's smaller dimension.
    pub fn rounded_outline(
        &mut self,
        pos: Vec2,
        size: Vec2,
        radius: f32,
        width: f32,
        color: Color,
    ) {
        if !pos.is_finite()
            || !size.is_finite()
            || !radius.is_finite()
            || !width.is_finite()
            || size.x <= 0.0
            || size.y <= 0.0
            || width <= 0.0
        {
            return;
        }
        let radius = radius.clamp(0.0, size.x.min(size.y) * 0.5);
        if radius <= f32::EPSILON {
            self.outline(pos, size, width, color);
            return;
        }

        let segments = (radius / 4.0).ceil().clamp(3.0, 12.0) as usize;
        let corners = [
            (pos + Vec2::new(size.x - radius, radius), -90.0_f32),
            (pos + size - Vec2::splat(radius), 0.0),
            (pos + Vec2::new(radius, size.y - radius), 90.0),
            (pos + Vec2::splat(radius), 180.0),
        ];

        let point = |center: Vec2, angle_degrees: f32| {
            let angle = angle_degrees.to_radians();
            center + Vec2::new(angle.cos(), angle.sin()) * radius
        };
        let mut previous = pos + Vec2::new(radius, 0.0);
        for (center, start_degrees) in corners {
            let arc_start = point(center, start_degrees);
            self.line(previous, arc_start, width, color);
            previous = arc_start;
            for step in 1..=segments {
                let fraction = step as f32 / segments as f32;
                let angle = start_degrees + 90.0 * fraction;
                let next = point(center, angle);
                self.line(previous, next, width, color);
                previous = next;
            }
        }
    }

    /// Add high-resolution text positioned by its top-left anchor.
    pub fn text(&mut self, pos: Vec2, text: impl Into<String>, font_size: f32, color: Color) {
        self.text.push(TextCommand {
            pos,
            text: text.into(),
            font_size: font_size.max(1.0),
            color: palette_color(color, self.palette),
        });
    }

    fn push_quad(&mut self, a: Vec2, b: Vec2, c: Vec2, d: Vec2, color: Color) {
        let base = self.vertices.len() as u32;
        let color = palette_color(color, self.palette)
            .to_linear()
            .to_f32_array();
        for point in [a, b, c, d] {
            self.vertices.push([
                point.x - self.size.x * 0.5,
                self.size.y * 0.5 - point.y,
                0.0,
            ]);
            self.colors.push(color);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn push_axis_quad(&mut self, a: Vec2, b: Vec2, color: Color) {
        self.push_quad(a, Vec2::new(b.x, a.y), b, Vec2::new(a.x, b.y), color);
    }
}

fn valid_size(size: Vec2) -> bool {
    size.is_finite() && size.x > 0.0 && size.y > 0.0
}

/// Post-processing controls for the pixel-art scene and CRT composite.
#[derive(Resource, Clone)]
pub struct RenderOptions {
    pub pixel: bool,
    /// Physical pixels per logical point on the primary display.
    pub scale_factor: f32,
    pub bloom: f32,
    pub scanlines: f32,
    pub crt: bool,
    pub curvature: f32,
    pub vignette: f32,
    pub chromatic_aberration: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            pixel: false,
            scale_factor: 1.0,
            bloom: 0.16,
            scanlines: 0.08,
            crt: true,
            curvature: 0.5,
            vignette: 0.45,
            chromatic_aberration: 0.35,
        }
    }
}

/// Renders batched geometry and high-resolution pooled text into separate
/// textures, then composites both through the CRT shader.
pub struct ArcadeRenderPlugin;

impl Plugin for ArcadeRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, CRT_SHADER, "../assets/crt.wgsl", Shader::from_wgsl);
        app.add_plugins(Material2dPlugin::<CrtMaterial>::default())
            .init_resource::<Frame>()
            .init_resource::<RenderOptions>()
            .add_systems(Startup, setup_renderer)
            .add_systems(Update, render_frame.after(crate::presentation::paint));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CrtMaterial {
    #[texture(0)]
    #[sampler(1)]
    source: Handle<Image>,
    #[uniform(2)]
    params: Vec4,
    #[texture(3)]
    #[sampler(4)]
    text_source: Handle<Image>,
    #[uniform(5)]
    monitor: Vec4,
}

impl Material2d for CrtMaterial {
    fn fragment_shader() -> ShaderRef {
        CRT_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }
}

#[derive(Resource)]
struct RendererState {
    scene_image: Handle<Image>,
    text_image: Handle<Image>,
    text_camera: Entity,
    last_text_size: UVec2,
    crt_material: Handle<CrtMaterial>,
    scene_mesh: Handle<Mesh>,
    scene_camera: Entity,
    composite_camera: Entity,
    composite_quad: Entity,
    text_entities: Vec<Entity>,
    last_scene_size: UVec2,
    last_logical_size: Vec2,
    last_pixel: bool,
}

#[derive(Component)]
struct SceneCamera;

#[derive(Component)]
struct CompositeCamera;

#[derive(Component)]
struct SceneGeometry;

#[derive(Component)]
struct CompositeQuad;

#[derive(Component)]
struct PooledText(usize);

fn setup_renderer(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut crt_materials: ResMut<Assets<CrtMaterial>>,
) {
    let initial_size = UVec2::new(1280, 720);
    let mut image = Image::new_target_texture(
        initial_size.x,
        initial_size.y,
        TextureFormat::Rgba8UnormSrgb,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_DST;
    image.sampler = ImageSampler::linear();
    let scene_image = images.add(image);
    let text_image = images.add(Image::new_target_texture(
        initial_size.x,
        initial_size.y,
        TextureFormat::Rgba8UnormSrgb,
    ));

    let scene_mesh = meshes.add(empty_triangle_mesh());
    let scene_material = color_materials.add(ColorMaterial::default());
    let crt_material = crt_materials.add(CrtMaterial {
        source: scene_image.clone(),
        params: Vec4::new(0.16, 0.08, 1.0, 0.0),
        text_source: text_image.clone(),
        monitor: Vec4::new(0.06, 0.45, 1.05, 0.0),
    });

    let scene_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -2,
                target: scene_image.clone().into(),
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: initial_size.x as f32,
                    height: initial_size.y as f32,
                },
                ..OrthographicProjection::default_2d()
            }),
            Msaa::Off,
            RenderLayers::layer(SCENE_LAYER),
            SceneCamera,
        ))
        .id();
    commands.spawn((
        SceneGeometry,
        Mesh2d(scene_mesh.clone()),
        MeshMaterial2d(scene_material.clone()),
        RenderLayers::layer(SCENE_LAYER),
    ));
    let text_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                target: text_image.clone().into(),
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: initial_size.x as f32,
                    height: initial_size.y as f32,
                },
                ..OrthographicProjection::default_2d()
            }),
            Msaa::Off,
            RenderLayers::layer(TEXT_LAYER),
        ))
        .id();
    let composite_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 0,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: initial_size.x as f32,
                    height: initial_size.y as f32,
                },
                ..OrthographicProjection::default_2d()
            }),
            RenderLayers::layer(COMPOSITE_LAYER),
            CompositeCamera,
        ))
        .id();
    let composite_quad = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(crt_material.clone()),
            Transform::from_xyz(0.0, 0.0, -100.0),
            RenderLayers::layer(COMPOSITE_LAYER),
            CompositeQuad,
        ))
        .id();

    commands.insert_resource(RendererState {
        scene_image,
        text_image,
        text_camera,
        last_text_size: initial_size,
        crt_material,
        scene_mesh,
        scene_camera,
        composite_camera,
        composite_quad,
        text_entities: Vec::new(),
        last_scene_size: initial_size,
        last_logical_size: initial_size.as_vec2(),
        last_pixel: false,
    });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn render_frame(
    mut commands: Commands,
    frame: Res<Frame>,
    options: Res<RenderOptions>,
    mut state: ResMut<RendererState>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut crt_materials: ResMut<Assets<CrtMaterial>>,
    mut projections: Query<&mut Projection>,
    mut transforms: Query<&mut Transform, Without<PooledText>>,
    mut text_query: Query<(
        &PooledText,
        &mut Text2d,
        &mut TextFont,
        &mut TextColor,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    if !valid_size(frame.size) {
        return;
    }

    let render_scale = if options.pixel {
        LOGICAL_PIXEL_SCALE
    } else {
        if options.scale_factor.is_finite() && options.scale_factor > 0.0 {
            options.scale_factor
        } else {
            1.0
        }
    };
    let width = (frame.size.x * render_scale).round().max(1.0) as u32;
    let height = (frame.size.y * render_scale).round().max(1.0) as u32;
    let scene_size = UVec2::new(width, height);
    let text_size = (frame.size * options.scale_factor.max(0.5))
        .round()
        .as_uvec2()
        .max(UVec2::ONE);
    let text_size_changed = text_size != state.last_text_size;
    if text_size_changed {
        if let Some(image) = images.get_mut(&state.text_image) {
            image.resize(Extent3d {
                width: text_size.x,
                height: text_size.y,
                depth_or_array_layers: 1,
            });
            image.sampler = ImageSampler::linear();
        }
        state.last_text_size = text_size;
    }
    let logical_size_changed = frame.size != state.last_logical_size;
    let pixel_mode_changed = options.pixel != state.last_pixel;
    if scene_size != state.last_scene_size
        || logical_size_changed
        || pixel_mode_changed
        || text_size_changed
    {
        if let Some(image) = images.get_mut(&state.scene_image) {
            if scene_size != state.last_scene_size {
                image.resize(Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                });
            }
            image.sampler = if options.pixel {
                ImageSampler::nearest()
            } else {
                ImageSampler::linear()
            };
        }
        for camera in [
            state.scene_camera,
            state.text_camera,
            state.composite_camera,
        ] {
            if let Ok(mut projection) = projections.get_mut(camera)
                && let Projection::Orthographic(orthographic) = &mut *projection
            {
                orthographic.scaling_mode = ScalingMode::Fixed {
                    width: frame.size.x,
                    height: frame.size.y,
                };
                orthographic.area = Rect::new(
                    -frame.size.x * 0.5,
                    -frame.size.y * 0.5,
                    frame.size.x * 0.5,
                    frame.size.y * 0.5,
                );
            }
        }
        state.last_scene_size = scene_size;
        state.last_logical_size = frame.size;
        state.last_pixel = options.pixel;
    }

    if let Some(mesh) = meshes.get_mut(&state.scene_mesh) {
        update_mesh(mesh, &frame);
    }
    if let Some(material) = crt_materials.get_mut(&state.crt_material) {
        let params = Vec4::new(
            options.bloom.max(0.0),
            options.scanlines.max(0.0),
            if options.crt { 1.0 } else { 0.0 },
            0.0,
        );
        if material.params != params {
            material.params = params;
        }
        let monitor = Vec4::new(
            if options.crt {
                options.curvature.clamp(0.0, 1.0) * 0.12
            } else {
                0.0
            },
            if options.crt {
                options.vignette.clamp(0.0, 1.0)
            } else {
                0.0
            },
            if options.crt {
                // At full strength, separate each outer channel by up to three
                // logical pixels at the edge, independent of DPI or preset.
                options.chromatic_aberration.clamp(0.0, 1.0) * 3.0 * options.scale_factor.max(0.5)
            } else {
                0.0
            },
            0.0,
        );
        if material.monitor != monitor {
            material.monitor = monitor;
        }
    }
    if let Ok(mut transform) = transforms.get_mut(state.composite_quad) {
        let scale = Vec3::new(frame.size.x, frame.size.y, 1.0);
        if transform.scale != scale {
            transform.scale = scale;
        }
    }

    while state.text_entities.len() < frame.text.len() {
        let index = state.text_entities.len();
        let entity = commands
            .spawn((
                PooledText(index),
                Text2d::new(""),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Anchor::TOP_LEFT,
                Transform::default(),
                Visibility::Hidden,
                RenderLayers::layer(TEXT_LAYER),
            ))
            .id();
        state.text_entities.push(entity);
    }

    for (index, mut text, mut font, mut color, mut transform, mut visibility) in &mut text_query {
        let Some(command) = frame.text.get(index.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if text.0 != command.text {
            text.0.clone_from(&command.text);
        }
        if (font.font_size - command.font_size).abs() > f32::EPSILON {
            font.font_size = command.font_size;
        }
        if color.0 != command.color {
            color.0 = command.color;
        }
        transform.translation = Vec3::new(
            command.pos.x - frame.size.x * 0.5,
            frame.size.y * 0.5 - command.pos.y,
            index.0 as f32,
        );
        *visibility = Visibility::Visible;
    }
}

fn empty_triangle_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

fn update_mesh(mesh: &mut Mesh, frame: &Frame) {
    if frame.vertices.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
        mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    } else {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, frame.vertices.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; frame.vertices.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, frame.colors.clone());
        mesh.insert_indices(Indices::U32(frame.indices.clone()));
    }
}

/// The same screen-to-surface transform used by the final fragment pass.
/// Bezel pixels outside the curved display deliberately have no interaction target.
pub fn screen_to_surface(point: Vec2, size: Vec2, curvature: f32) -> Option<Vec2> {
    if !valid_size(size) || !point.is_finite() {
        return None;
    }
    let p = point / size * 2.0 - Vec2::ONE;
    let q = p * (1.0 + curvature.clamp(0.0, 1.0) * 0.12 * p.length_squared());
    let surface = (q + Vec2::ONE) * 0.5 * size;
    (surface.cmpge(Vec2::ZERO).all() && surface.cmplt(size).all()).then_some(surface)
}

fn palette_color(color: Color, scheme: gamevex_core::ColorScheme) -> Color {
    if scheme == gamevex_core::ColorScheme::Cyan {
        return color;
    }
    let c = color.to_srgba();
    let peak = c.red.max(c.green).max(c.blue);
    let rgb = if c.red > 0.7 && c.green > 0.7 && c.blue > 0.7 {
        [peak, peak * 0.92, peak * 0.72]
    } else if c.red > 0.8 && c.green < 0.35 && c.blue < 0.5 {
        [1.0, 0.33, 0.18] // Keep losses and wrong flags distinct from amber accents.
    } else if c.red > 0.8 && c.green < 0.3 && c.blue > 0.5 {
        [1.0, 0.53, 0.16]
    } else if c.red > 0.8 && c.green > 0.5 && c.blue < 0.2 {
        [1.0, 0.88, 0.56]
    } else {
        [peak, peak * 0.73, peak * 0.26]
    };
    Color::srgba(rgb[0], rgb[1], rgb[2], c.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monitor_picking_matches_shader_at_center_edges_and_resizes() {
        for size in [Vec2::new(1280.0, 800.0), Vec2::new(390.0, 844.0)] {
            for strength in [0.0, 0.5, 1.0] {
                for target in [
                    size * 0.5,
                    Vec2::new(22.0, 30.0),
                    size - Vec2::new(22.0, 40.0),
                ] {
                    let q = target / size * 2.0 - Vec2::ONE;
                    let k = strength * 0.12;
                    let mut p = q;
                    for _ in 0..30 {
                        p = q / (1.0 + k * p.length_squared());
                    }
                    let screen = (p + Vec2::ONE) * 0.5 * size;
                    assert!(
                        screen_to_surface(screen, size, strength)
                            .unwrap()
                            .distance(target)
                            < 0.01
                    );
                }
            }
            assert!(screen_to_surface(Vec2::ZERO, size, 0.5).is_none());
            assert_eq!(screen_to_surface(size * 0.5, size, 1.0), Some(size * 0.5));
        }
    }
}
