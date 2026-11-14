//! Small Web Audio playback adapter for browser builds.
//!
//! Bevy's web audio backend can remain suspended until a real user gesture.
//! This module creates and resumes one context directly in a canvas gesture
//! handler, then plays the app's short PCM cues from cached AudioBuffers.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub running: bool,
    pub voices: usize,
    pub cached: usize,
}

pub use wasm::{install, play, status, update};

const SAMPLE_RATE: u32 = 22_050;
const MAX_WAV_BYTES: usize = 2 * 1024 * 1024;
const MAX_CONTEXT_SAMPLE_RATE: u32 = 384_000;

/// Returns validated PCM16 samples before conversion to Web Audio's float format.
fn pcm16_mono_values(wav: &[u8]) -> Option<Vec<i16>> {
    if wav.len() < 12 || wav.len() > MAX_WAV_BYTES || &wav[..4] != b"RIFF" || &wav[8..12] != b"WAVE"
    {
        return None;
    }

    let mut format = None;
    let mut data = None;
    let mut offset = 12usize;
    while offset.checked_add(8)? <= wav.len() {
        let id = &wav[offset..offset + 4];
        let size = u32::from_le_bytes(wav[offset + 4..offset + 8].try_into().ok()?) as usize;
        let start = offset.checked_add(8)?;
        let end = start.checked_add(size)?;
        if end > wav.len() {
            return None;
        }
        if id == b"fmt " {
            if size < 16 {
                return None;
            }
            format = Some((
                u16::from_le_bytes(wav[start..start + 2].try_into().ok()?),
                u16::from_le_bytes(wav[start + 2..start + 4].try_into().ok()?),
                u32::from_le_bytes(wav[start + 4..start + 8].try_into().ok()?),
                u16::from_le_bytes(wav[start + 12..start + 14].try_into().ok()?),
                u16::from_le_bytes(wav[start + 14..start + 16].try_into().ok()?),
            ));
        } else if id == b"data" {
            data = Some((start, end));
        }
        offset = end.checked_add(size & 1)?;
    }

    let (encoding, channels, sample_rate, block_align, bits_per_sample) = format?;
    if encoding != 1
        || channels != 1
        || sample_rate != SAMPLE_RATE
        || block_align != 2
        || bits_per_sample != 16
    {
        return None;
    }
    let (start, end) = data?;
    let pcm = &wav[start..end];
    if pcm.is_empty() || !pcm.len().is_multiple_of(2) {
        return None;
    }
    Some(
        pcm.as_chunks::<2>()
            .0
            .iter()
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]))
            .collect(),
    )
}

/// Resamples like rodio 0.20's `UniformSourceIterator` for mono PCM16 input.
/// Rodio interpolates in the integer sample domain, truncating each lerp toward zero.
fn resample_pcm16(samples: &[i16], source_rate: u32, target_rate: u32) -> Option<Vec<i16>> {
    if samples.is_empty()
        || source_rate == 0
        || target_rate == 0
        || target_rate > MAX_CONTEXT_SAMPLE_RATE
    {
        return None;
    }
    if source_rate == target_rate {
        return Some(samples.to_vec());
    }

    let divisor = gcd(source_rate, target_rate);
    let from = source_rate / divisor;
    let to = target_rate / divisor;
    // Generated cues and legacy tones are far shorter than this; keep this
    // conversion bounded for malformed or unexpectedly large WAV data.
    if samples.len() > 32_768 {
        return None;
    }
    let capacity = samples
        .len()
        .checked_mul(to as usize)?
        .checked_add(from as usize - 1)?
        / from as usize;
    let mut output = Vec::with_capacity(capacity);
    let mut output_frame = 0u64;
    loop {
        let source_position = output_frame.checked_mul(from as u64)?;
        let left_index = (source_position / to as u64) as usize;
        if left_index >= samples.len() {
            break;
        }
        let left = samples[left_index];
        let Some(&right) = samples.get(left_index + 1) else {
            // rodio drains the final current frame unchanged when the next
            // interpolation frame is unavailable.
            output.push(left);
            break;
        };
        let numerator = (source_position % to as u64) as i32;
        let denominator = to as i32;
        let value = left as i32 + (right as i32 - left as i32) * numerator / denominator;
        output.push(value as i16);
        output_frame += 1;
    }
    Some(output)
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

mod wasm {
    use super::{Status, pcm16_mono_values, resample_pcm16};
    use std::{cell::RefCell, collections::HashMap};
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::{
        AudioBuffer, AudioBufferSourceNode, AudioContext, AudioContextState, GainNode,
        HtmlCanvasElement,
    };

    use crate::MAX_AUDIO_VOICES as MAX_VOICES;
    const MAX_CACHED_CUES: usize = 21;

    struct Voice {
        source: AudioBufferSourceNode,
        gain: GainNode,
        base_gain: f32,
        ends_at: f64,
        priority: u8,
        casino: bool,
    }

    #[derive(Default)]
    struct Audio {
        context: Option<AudioContext>,
        buffers: HashMap<u32, AudioBuffer>,
        buffer_order: Vec<u32>,
        voices: Vec<Voice>,
        volume: f32,
        muted: bool,
        active_casino: bool,
    }

    type EventListener = Closure<dyn FnMut(web_sys::Event)>;

    thread_local! {
        static AUDIO: RefCell<Audio> = RefCell::new(Audio {
            volume: 1.0,
            active_casino: true,
            ..Audio::default()
        });
        static LISTENERS: RefCell<Vec<EventListener>> = const { RefCell::new(Vec::new()) };
    }

    pub fn install() {
        if LISTENERS.with(|listeners| !listeners.borrow().is_empty()) {
            return;
        }
        let Some(canvas) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("gamevex"))
            .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return;
        };

        let mut installed = Vec::new();
        for event_name in ["pointerdown", "keydown"] {
            let callback = Closure::<dyn FnMut(web_sys::Event)>::new(|_| {
                // Construct and resume synchronously from the trusted gesture.
                ensure_context_and_resume();
            });
            if canvas
                .add_event_listener_with_callback(event_name, callback.as_ref().unchecked_ref())
                .is_ok()
            {
                installed.push(callback);
            }
        }
        LISTENERS.with(|listeners| listeners.borrow_mut().extend(installed));
    }

    fn ensure_context_and_resume() {
        AUDIO.with(|audio| {
            let mut audio = audio.borrow_mut();
            if audio.context.is_none() {
                audio.context = AudioContext::new().ok();
            }
            if let Some(context) = audio.context.as_ref()
                && context.state() == AudioContextState::Suspended
            {
                // Browser policy resolves/rejects this promise asynchronously.
                // A later cue can play after it becomes Running; cues are never queued.
                let _ = context.resume();
            }
        });
    }

    pub fn update(volume: f32, muted: bool, active_casino: bool) {
        AUDIO.with(|audio| {
            let mut audio = audio.borrow_mut();
            audio.volume = finite_gain(volume);
            audio.muted = muted;
            audio.active_casino = active_casino;
            if muted || audio.volume == 0.0 {
                stop_all(&mut audio.voices);
                return;
            }
            let master = audio.volume;
            let casino_active = audio.active_casino;
            for voice in &audio.voices {
                let category_gain = if voice.casino && !casino_active {
                    0.0
                } else {
                    1.0
                };
                voice
                    .gain
                    .gain()
                    .set_value(voice.base_gain * master * category_gain);
            }
            let now = audio
                .context
                .as_ref()
                .map(AudioContext::current_time)
                .unwrap_or(0.0);
            audio.voices.retain_mut(|voice| {
                if voice.ends_at <= now || (voice.casino && !casino_active) {
                    dispose(voice);
                    false
                } else {
                    true
                }
            });
        });
    }

    pub fn play(key: u32, wav: &[u8], gain: f32, priority: u8, casino: bool) -> bool {
        AUDIO.with(|audio| {
            let mut audio = audio.borrow_mut();
            prune_expired(&mut audio);
            let Some(context) = audio.context.clone() else {
                return false;
            };
            if context.state() != AudioContextState::Running || audio.muted || audio.volume == 0.0 {
                return false;
            }

            if audio.voices.len() >= MAX_VOICES {
                let (index, lowest) = audio
                    .voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, voice)| voice.priority)
                    .map(|(index, voice)| (index, voice.priority))
                    .unwrap();
                if lowest >= priority {
                    return false;
                }
                let mut evicted = audio.voices.remove(index);
                dispose(&mut evicted);
            }

            let target_rate = context.sample_rate().round() as u32;
            if target_rate == 0 || target_rate > super::MAX_CONTEXT_SAMPLE_RATE {
                return false;
            }
            let buffer = if let Some(buffer) = audio.buffers.get(&key) {
                buffer.clone()
            } else {
                let Some(samples) = pcm16_mono_values(wav) else {
                    return false;
                };
                let Some(samples) = resample_pcm16(&samples, super::SAMPLE_RATE, target_rate)
                else {
                    return false;
                };
                let samples: Vec<f32> = samples
                    .into_iter()
                    .map(|sample| sample as f32 / 32768.0)
                    .collect();
                let Ok(buffer) = context.create_buffer(1, samples.len() as u32, target_rate as f32)
                else {
                    return false;
                };
                if buffer.copy_to_channel(&samples, 0).is_err() {
                    return false;
                }
                if audio.buffers.len() >= MAX_CACHED_CUES
                    && let Some(oldest) = audio.buffer_order.first().copied()
                {
                    audio.buffer_order.remove(0);
                    audio.buffers.remove(&oldest);
                }
                audio.buffer_order.push(key);
                audio.buffers.insert(key, buffer.clone());
                buffer
            };

            let Ok(source) = context.create_buffer_source() else {
                return false;
            };
            let Ok(gain_node) = context.create_gain() else {
                return false;
            };
            source.set_buffer(Some(&buffer));
            let base_gain = finite_gain(gain);
            let category_gain = if casino && !audio.active_casino {
                0.0
            } else {
                1.0
            };
            gain_node
                .gain()
                .set_value(base_gain * audio.volume * category_gain);
            if source.connect_with_audio_node(&gain_node).is_err()
                || gain_node
                    .connect_with_audio_node(&context.destination())
                    .is_err()
            {
                let _ = source.disconnect();
                let _ = gain_node.disconnect();
                return false;
            }
            if source.start().is_err() {
                let _ = source.disconnect();
                let _ = gain_node.disconnect();
                return false;
            }
            audio.voices.push(Voice {
                source,
                gain: gain_node,
                base_gain,
                ends_at: context.current_time() + buffer.duration(),
                priority,
                casino,
            });
            true
        })
    }

    pub fn status() -> Status {
        AUDIO.with(|audio| {
            let mut audio = audio.borrow_mut();
            prune_expired(&mut audio);
            Status {
                running: audio
                    .context
                    .as_ref()
                    .map(|context| context.state() == AudioContextState::Running)
                    .unwrap_or(false),
                voices: audio.voices.len(),
                cached: audio.buffers.len(),
            }
        })
    }

    fn prune_expired(audio: &mut Audio) {
        let now = audio
            .context
            .as_ref()
            .map(AudioContext::current_time)
            .unwrap_or(0.0);
        audio.voices.retain_mut(|voice| {
            if voice.ends_at <= now {
                dispose(voice);
                false
            } else {
                true
            }
        });
    }

    fn stop_all(voices: &mut Vec<Voice>) {
        for voice in voices.iter_mut() {
            dispose(voice);
        }
        voices.clear();
    }

    fn dispose(voice: &mut Voice) {
        let scheduled: &web_sys::AudioScheduledSourceNode = voice.source.unchecked_ref();
        let _ = scheduled.stop();
        let _ = voice.source.disconnect();
        let _ = voice.gain.disconnect();
    }

    fn finite_gain(gain: f32) -> f32 {
        if gain.is_finite() {
            gain.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pcm16_mono_values;

    fn wav(samples: &[i16]) -> Vec<u8> {
        let data_len = (samples.len() * 2) as u32;
        let mut bytes = Vec::with_capacity(44 + data_len as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&22_050u32.to_le_bytes());
        bytes.extend_from_slice(&44_100u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn parses_expected_pcm_samples_and_rejects_malformed_wavs() {
        assert_eq!(
            pcm16_mono_values(&wav(&[0, i16::MAX, i16::MIN])),
            Some(vec![0, i16::MAX, i16::MIN])
        );
        assert_eq!(pcm16_mono_values(b"RIFF"), None);
        assert_eq!(pcm16_mono_values(&wav(&[])), None);
        let mut wrong_rate = wav(&[1]);
        wrong_rate[24..28].copy_from_slice(&44_100u32.to_le_bytes());
        assert_eq!(pcm16_mono_values(&wrong_rate), None);
    }
}
