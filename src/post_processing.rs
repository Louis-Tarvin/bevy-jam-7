use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};

use crate::game::camera::MainCamera;

const SHADER_ASSET_PATH: &str = "shaders/cloud_vignette.wgsl";

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DreamCloudVignette>();
    app.add_plugins((
        ExtractComponentPlugin::<DreamCloudPostProcessSettings>::default(),
        UniformComponentPlugin::<DreamCloudPostProcessSettings>::default(),
    ));
    app.add_systems(
        Update,
        (
            attach_post_process_to_main_camera,
            animate_cloud_coverage,
            sync_settings_from_resource,
        )
            .chain(),
    );

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app.add_systems(RenderStartup, init_post_process_pipeline);
    render_app
        .add_render_graph_node::<ViewNodeRunner<DreamCloudNode>>(Core3d, DreamCloudLabel)
        .add_render_graph_edges(
            Core3d,
            (
                Node3d::Tonemapping,
                DreamCloudLabel,
                Node3d::EndMainPassPostProcessing,
            ),
        );
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct DreamCloudVignette {
    pub coverage: f32,
    pub target_coverage: f32,
    pub transition_speed: f32,
    pub edge_softness: f32,
    pub boundary_thickness: f32,
    pub wobble_strength: f32,
    pub wobble_frequency: f32,
    pub wobble_speed: f32,
}

impl Default for DreamCloudVignette {
    fn default() -> Self {
        Self {
            coverage: 1.0,
            target_coverage: 0.2,
            transition_speed: 3.0,
            edge_softness: 0.03,
            boundary_thickness: 0.08,
            wobble_strength: 0.045,
            wobble_frequency: 8.0,
            wobble_speed: 2.0,
        }
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct DreamCloudPostProcessSettings {
    // x = coverage, y = time, z = edge_softness, w = boundary_thickness
    boundary: Vec4,
    // x = wobble_strength, y = wobble_frequency, z = wobble_speed
    wobble: Vec4,
}

impl Default for DreamCloudPostProcessSettings {
    fn default() -> Self {
        Self {
            boundary: Vec4::new(0.16, 0.0, 0.03, 0.08),
            wobble: Vec4::new(0.045, 8.0, 2.0, 0.0),
        }
    }
}

fn attach_post_process_to_main_camera(
    mut commands: Commands,
    cameras: Query<Entity, (With<MainCamera>, Without<DreamCloudPostProcessSettings>)>,
) {
    for entity in &cameras {
        commands
            .entity(entity)
            .insert(DreamCloudPostProcessSettings::default());
    }
}

fn animate_cloud_coverage(time: Res<Time>, mut vignette: ResMut<DreamCloudVignette>) {
    let target = vignette.target_coverage.clamp(0.0, 1.0);
    let speed = vignette.transition_speed.max(0.0);

    if speed == 0.0 {
        vignette.coverage = target;
        return;
    }

    let t = 1.0 - (-speed * time.delta_secs()).exp();
    vignette.coverage += (target - vignette.coverage) * t;
    if (target - vignette.coverage).abs() < 0.001 {
        vignette.coverage = target;
    }
}

fn sync_settings_from_resource(
    time: Res<Time>,
    vignette: Res<DreamCloudVignette>,
    mut settings: Query<&mut DreamCloudPostProcessSettings, With<MainCamera>>,
) {
    let coverage = vignette.coverage.clamp(0.0, 1.0);
    let edge_softness = vignette.edge_softness.max(0.001);
    let boundary_thickness = vignette.boundary_thickness.max(0.0);
    let wobble_strength = vignette.wobble_strength.max(0.0);
    let wobble_frequency = vignette.wobble_frequency.max(0.0);
    let wobble_speed = vignette.wobble_speed;

    for mut post_process in &mut settings {
        post_process.boundary = Vec4::new(
            coverage,
            time.elapsed_secs(),
            edge_softness,
            boundary_thickness,
        );
        post_process.wobble = Vec4::new(wobble_strength, wobble_frequency, wobble_speed, 0.0);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct DreamCloudLabel;

#[derive(Default)]
struct DreamCloudNode;

impl ViewNode for DreamCloudNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static DreamCloudPostProcessSettings,
        &'static DynamicUniformIndex<DreamCloudPostProcessSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<DreamCloudPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms =
            world.resource::<ComponentUniforms<DreamCloudPostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            "dream_cloud_bind_group",
            &pipeline_cache.get_bind_group_layout(&post_process_pipeline.layout),
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("dream_cloud_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);
        Ok(())
    }
}

#[derive(Resource)]
struct DreamCloudPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_post_process_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "dream_cloud_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<DreamCloudPostProcessSettings>(true),
            ),
        ),
    );
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let shader = asset_server.load(SHADER_ASSET_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("dream_cloud_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(DreamCloudPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}
