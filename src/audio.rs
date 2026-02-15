use bevy::{audio::Volume, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<BgmConfig>();
    app.add_systems(
        Update,
        (
            apply_global_volume.run_if(resource_changed::<GlobalVolume>),
            bgm_config_changed.run_if(resource_changed::<BgmConfig>),
        ),
    );
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Music;

#[derive(Debug, Component)]
pub enum MusicLayer {
    Base,
    Extra,
    Perc,
}

/// A music audio instance.
pub fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::LOOP, Music)
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BgmConfig {
    pub base_enabled: bool,
    pub extra_enabled: bool,
    pub percussion_enabled: bool,
}

fn bgm_config_changed(
    config: Res<BgmConfig>,
    global_volume: Res<GlobalVolume>,
    query: Query<(&mut AudioSink, &MusicLayer)>,
) {
    for (mut sink, layer) in query {
        match layer {
            MusicLayer::Base => {
                if config.base_enabled {
                    sink.set_volume(global_volume.volume);
                } else {
                    sink.set_volume(Volume::SILENT);
                }
            }
            MusicLayer::Extra => {
                if config.extra_enabled {
                    sink.set_volume(global_volume.volume);
                } else {
                    sink.set_volume(Volume::SILENT);
                }
            }
            MusicLayer::Perc => {
                if config.percussion_enabled {
                    sink.set_volume(global_volume.volume);
                } else {
                    sink.set_volume(Volume::SILENT);
                }
            }
        }
    }
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SoundEffect;

/// A sound effect audio instance.
pub fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::DESPAWN, SoundEffect)
}

pub fn sound_effect_3d(handle: Handle<AudioSource>, translation: Vec3) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            spatial: true,
            volume: Volume::Linear(0.8),
            ..PlaybackSettings::ONCE
        },
        SoundEffect,
        Transform::from_translation(translation),
    )
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    global_volume: Res<GlobalVolume>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink)>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_volume(global_volume.volume * playback.volume);
    }
}
