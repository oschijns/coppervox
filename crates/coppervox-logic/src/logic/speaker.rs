/*
use bevy_kira_audio::prelude::*;

/// Read the input as a pitch and play the corresponding sound
#[derive(Component)]
pub struct Speaker {
    /// handle to the sound to play
    audio_handle: Handle<AudioInstance>,

    /// audio tween when starting the sound
    audio_tween: AudioTween,

    /// has the signal being cut during a frame
    was_cut: bool,
}

/// Used to construct a constant gate
#[derive(Bundle)]
pub struct SpeakerBundle<R: Element> {
    render: Render<R>,
    speaker: Speaker,
    inputs: GateInputs,
}

/// For each constant gate in the simulation,
/// output its value.
pub fn process<
    R: Element + for<'a> From<&'a V>,
    V: Element + Copy + Value + Nillable,
    C: Element + Copy,
>(
    mut speaker_query: Query<(&mut Render<R>, &mut Speaker, &GateInputs)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    wire_query: Query<&WireState<V, C>>,
) {
    // for each speaker
    for (mut render, mut speaker, ref_inputs) in speaker_query.iter_mut() {
        // combine all of the inputs into a single value
        let mut value = V::default();

        // for each wire input references
        for &id in ref_inputs.wire_references.iter() {
            if let Ok(wire) = wire_query.get(id) {
                value.fuse_with(&wire.get_value());
            }
        }

        // refresh the rendering of the component
        render.refresh(&value);

        // check if the sound should be restarted
        if value.is_nil() {
            speaker.was_cut = true;
        } else if speaker.was_cut {
            speaker.was_cut = false;

            // If the sound was cut during the frame,
            // and it should be played now, restart it.
            if let Some(audio) = audio_instances.get_mut(&speaker.audio_handle) {
                audio.seek_to(0.0);
                audio.resume(speaker.audio_tween.clone());
            }
        }
    }
}

*/
