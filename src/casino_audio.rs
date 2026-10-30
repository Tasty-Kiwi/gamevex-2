//! Small, deterministic synthesized cues for casino, patience, and tile games.
//!
//! The sounds are rendered to a compact PCM WAV so callers can cache them as
//! ordinary Bevy audio assets. No audio or random-number dependencies needed.

const SAMPLE_RATE: u32 = 22_050;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SoundCue {
    Bet,
    Deal,
    Card,
    Reveal,
    Hit,
    Stand,
    Double,
    Split,
    Surrender,
    Insurance,
    Bust,
    Win,
    Natural,
    Push,
    Loss,
    GameOver,
    TablePick,
    TableMove,
    TableDeal,
    TableUndo,
    TableHint,
    TableShuffle,
    TableInvalid,
    TableRun,
    TableWin,
    TilePick,
    TileMatch,
}

pub const ALL: [SoundCue; 27] = SoundCue::ALL;

impl SoundCue {
    pub const ALL: [Self; 27] = [
        Self::Bet,
        Self::Deal,
        Self::Card,
        Self::Reveal,
        Self::Hit,
        Self::Stand,
        Self::Double,
        Self::Split,
        Self::Surrender,
        Self::Insurance,
        Self::Bust,
        Self::Win,
        Self::Natural,
        Self::Push,
        Self::Loss,
        Self::GameOver,
        Self::TablePick,
        Self::TableMove,
        Self::TableDeal,
        Self::TableUndo,
        Self::TableHint,
        Self::TableShuffle,
        Self::TableInvalid,
        Self::TableRun,
        Self::TableWin,
        Self::TilePick,
        Self::TileMatch,
    ];
}

#[derive(Clone, Copy)]
enum Voice {
    Sine,
    Bell,
    Noise,
    Thud,
}

#[derive(Clone, Copy)]
struct Note {
    start: f32,
    length: f32,
    pitch: f32,
    end_pitch: f32,
    level: f32,
    voice: Voice,
}

const fn note(start: f32, length: f32, pitch: f32, level: f32, voice: Voice) -> Note {
    Note {
        start,
        length,
        pitch,
        end_pitch: pitch,
        level,
        voice,
    }
}

const fn sweep(
    start: f32,
    length: f32,
    pitch: f32,
    end_pitch: f32,
    level: f32,
    voice: Voice,
) -> Note {
    Note {
        start,
        length,
        pitch,
        end_pitch,
        level,
        voice,
    }
}

/// Synthesize one restrained retro-casino cue as 22050 Hz, mono, PCM16 WAV.
pub fn synthesize(cue: SoundCue) -> Vec<u8> {
    use SoundCue as C;
    use Voice::{Bell, Noise, Sine, Thud};

    let (duration, notes): (f32, &[Note]) = match cue {
        C::Bet => (
            0.19,
            &[
                note(0.000, 0.045, 0.0, 0.22, Noise),
                note(0.008, 0.115, 1160.0, 0.20, Bell),
                note(0.048, 0.075, 760.0, 0.09, Bell),
            ],
        ),
        C::Deal => (
            0.31,
            &[
                sweep(0.000, 0.220, 2100.0, 520.0, 0.18, Noise),
                sweep(0.000, 0.150, 310.0, 185.0, 0.13, Thud),
                note(0.170, 0.095, 590.0, 0.11, Sine),
            ],
        ),
        C::Card => (
            0.27,
            &[
                sweep(0.000, 0.195, 2600.0, 980.0, 0.17, Noise),
                note(0.115, 0.090, 430.0, 0.09, Sine),
            ],
        ),
        C::Reveal => (
            0.48,
            &[
                note(0.000, 0.190, 660.0, 0.14, Bell),
                note(0.105, 0.210, 990.0, 0.13, Bell),
                note(0.220, 0.240, 1320.0, 0.12, Bell),
                note(0.225, 0.180, 1980.0, 0.035, Bell),
            ],
        ),
        C::Hit => (
            0.22,
            &[
                note(0.000, 0.065, 0.0, 0.19, Noise),
                note(0.005, 0.135, 250.0, 0.17, Thud),
            ],
        ),
        C::Stand => (
            0.16,
            &[
                note(0.000, 0.032, 0.0, 0.13, Noise),
                note(0.000, 0.115, 720.0, 0.13, Sine),
            ],
        ),
        C::Double => (
            0.39,
            &[
                note(0.000, 0.145, 590.0, 0.15, Bell),
                note(0.145, 0.210, 880.0, 0.16, Bell),
                note(0.145, 0.140, 1760.0, 0.035, Bell),
            ],
        ),
        C::Split => (
            0.38,
            &[
                note(0.000, 0.130, 520.0, 0.15, Bell),
                note(0.105, 0.180, 780.0, 0.15, Bell),
                note(0.105, 0.120, 1560.0, 0.035, Bell),
            ],
        ),
        C::Surrender => (
            0.48,
            &[
                note(0.000, 0.190, 560.0, 0.13, Bell),
                note(0.175, 0.230, 420.0, 0.12, Bell),
                note(0.175, 0.160, 630.0, 0.035, Bell),
            ],
        ),
        C::Insurance => (
            0.42,
            &[
                note(0.000, 0.185, 784.0, 0.13, Bell),
                note(0.115, 0.220, 1175.0, 0.12, Bell),
            ],
        ),
        C::Bust => (
            0.52,
            &[
                note(0.000, 0.085, 0.0, 0.21, Noise),
                sweep(0.000, 0.330, 185.0, 92.0, 0.21, Thud),
                sweep(0.000, 0.240, 330.0, 160.0, 0.08, Sine),
            ],
        ),
        C::Win => (
            0.77,
            &[
                note(0.000, 0.190, 523.0, 0.12, Bell),
                note(0.135, 0.190, 659.0, 0.12, Bell),
                note(0.270, 0.220, 784.0, 0.12, Bell),
                note(0.405, 0.320, 1047.0, 0.13, Bell),
                note(0.405, 0.230, 1568.0, 0.035, Bell),
            ],
        ),
        C::Natural => (
            1.20,
            &[
                note(0.000, 0.280, 523.0, 0.115, Bell),
                note(0.165, 0.290, 659.0, 0.115, Bell),
                note(0.330, 0.300, 784.0, 0.120, Bell),
                note(0.495, 0.590, 1047.0, 0.125, Bell),
                note(0.495, 0.400, 1319.0, 0.055, Bell),
                note(0.495, 0.310, 1568.0, 0.035, Bell),
            ],
        ),
        C::Push => (
            0.58,
            &[
                note(0.000, 0.260, 659.0, 0.11, Bell),
                note(0.180, 0.300, 659.0, 0.10, Bell),
            ],
        ),
        C::Loss => (
            0.61,
            &[
                note(0.000, 0.230, 494.0, 0.105, Bell),
                note(0.180, 0.240, 392.0, 0.095, Bell),
                note(0.360, 0.230, 330.0, 0.090, Bell),
            ],
        ),
        C::GameOver => (
            0.92,
            &[
                note(0.000, 0.300, 392.0, 0.11, Bell),
                note(0.220, 0.320, 330.0, 0.105, Bell),
                note(0.460, 0.390, 262.0, 0.10, Bell),
                note(0.460, 0.280, 131.0, 0.075, Sine),
            ],
        ),
        // Board game interactions use quiet, woody clicks and small warm chimes.
        C::TablePick => (
            0.16,
            &[
                note(0.000, 0.055, 0.0, 0.14, Noise),
                note(0.000, 0.105, 440.0, 0.10, Thud),
                note(0.018, 0.095, 880.0, 0.045, Bell),
            ],
        ),
        C::TableMove => (
            0.20,
            &[
                note(0.000, 0.045, 0.0, 0.10, Noise),
                note(0.006, 0.095, 330.0, 0.09, Thud),
                note(0.045, 0.100, 550.0, 0.055, Bell),
            ],
        ),
        // Card tables have a soft paper swish followed by a muted placement tap.
        C::TableDeal => (
            0.31,
            &[
                sweep(0.000, 0.190, 2300.0, 820.0, 0.13, Noise),
                sweep(0.025, 0.130, 420.0, 250.0, 0.07, Thud),
                note(0.190, 0.080, 280.0, 0.10, Thud),
            ],
        ),
        C::TableUndo => (
            0.25,
            &[
                sweep(0.000, 0.105, 620.0, 390.0, 0.08, Bell),
                note(0.100, 0.100, 390.0, 0.075, Thud),
                note(0.095, 0.100, 520.0, 0.04, Bell),
            ],
        ),
        C::TableHint => (
            0.30,
            &[
                note(0.000, 0.115, 660.0, 0.075, Bell),
                note(0.135, 0.135, 990.0, 0.08, Bell),
            ],
        ),
        C::TableShuffle => (
            0.42,
            &[
                sweep(0.000, 0.240, 2500.0, 900.0, 0.11, Noise),
                sweep(0.100, 0.210, 1900.0, 700.0, 0.085, Noise),
                note(0.310, 0.080, 350.0, 0.08, Thud),
            ],
        ),
        C::TableInvalid => (
            0.30,
            &[
                sweep(0.000, 0.210, 520.0, 300.0, 0.075, Bell),
                sweep(0.020, 0.180, 260.0, 190.0, 0.035, Sine),
            ],
        ),
        C::TableRun => (
            0.62,
            &[
                note(0.000, 0.150, 440.0, 0.085, Bell),
                note(0.125, 0.160, 554.0, 0.085, Bell),
                note(0.250, 0.180, 659.0, 0.09, Bell),
                note(0.375, 0.230, 880.0, 0.10, Bell),
            ],
        ),
        C::TableWin => (
            0.92,
            &[
                note(0.000, 0.240, 523.0, 0.09, Bell),
                note(0.115, 0.260, 659.0, 0.085, Bell),
                note(0.230, 0.300, 784.0, 0.09, Bell),
                note(0.345, 0.490, 1047.0, 0.10, Bell),
                note(0.345, 0.380, 1319.0, 0.05, Bell),
                note(0.345, 0.300, 1568.0, 0.035, Bell),
            ],
        ),
        // Mahjong-like tile feedback: two dry wood taps, then a paired chime.
        C::TilePick => (
            0.18,
            &[
                note(0.000, 0.045, 0.0, 0.13, Noise),
                note(0.000, 0.105, 260.0, 0.13, Thud),
                note(0.000, 0.080, 1040.0, 0.045, Bell),
            ],
        ),
        C::TileMatch => (
            0.40,
            &[
                note(0.000, 0.055, 0.0, 0.12, Noise),
                note(0.000, 0.110, 280.0, 0.12, Thud),
                note(0.080, 0.055, 0.0, 0.10, Noise),
                note(0.080, 0.115, 340.0, 0.10, Thud),
                note(0.145, 0.145, 880.0, 0.075, Bell),
                note(0.245, 0.145, 1320.0, 0.07, Bell),
            ],
        ),
    };

    render(duration, notes, cue as u32)
}

fn render(duration: f32, notes: &[Note], seed: u32) -> Vec<u8> {
    let count = (duration * SAMPLE_RATE as f32).ceil() as usize;
    let data_bytes = count * 2;
    let mut wav = Vec::with_capacity(data_bytes + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36u32 + data_bytes as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_bytes as u32).to_le_bytes());

    let mut phases = [0.0f32; 8];
    for frame in 0..count {
        let time = frame as f32 / SAMPLE_RATE as f32;
        let mut sample = 0.0f32;
        for (index, note) in notes.iter().enumerate() {
            let local = time - note.start;
            if local < 0.0 || local >= note.length {
                continue;
            }
            let progress = local / note.length;
            // Smooth the onset and the end of every component to avoid clicks.
            let attack = (local / 0.006).clamp(0.0, 1.0);
            let release = ((note.length - local) / 0.035).clamp(0.0, 1.0);
            let envelope = attack * attack * release * release;
            let pitch = note.pitch + (note.end_pitch - note.pitch) * progress;
            let signal = match note.voice {
                Voice::Noise => {
                    let noise = white_noise(frame as u32, seed.wrapping_add(index as u32 * 97));
                    // Differencing makes the deterministic noise feel like a light swish.
                    let previous = white_noise(
                        frame.saturating_sub(1) as u32,
                        seed.wrapping_add(index as u32 * 97),
                    );
                    (noise - previous) * 0.48
                }
                _ => {
                    phases[index] += std::f32::consts::TAU * pitch.max(1.0) / SAMPLE_RATE as f32;
                    let phase = phases[index];
                    match note.voice {
                        Voice::Sine => phase.sin(),
                        Voice::Bell => {
                            // Inharmonic partials give the chime a soft, old-casino color.
                            (phase.sin()
                                + (phase * 2.71).sin() * 0.22
                                + (phase * 4.08).sin() * 0.08)
                                * (1.0 - progress * 0.22)
                        }
                        Voice::Thud => phase.sin() * (1.0 - progress * 0.72),
                        Voice::Noise => unreachable!(),
                    }
                }
            };
            sample += signal * note.level * envelope;
        }
        // Fixed gain ceiling leaves headroom for mixers and makes clipping impossible.
        let pcm = (sample.clamp(-0.62, 0.62) * i16::MAX as f32) as i16;
        wav.extend_from_slice(&pcm.to_le_bytes());
    }
    wav
}

fn white_noise(frame: u32, seed: u32) -> f32 {
    let mut value = frame
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^= value >> 16;
    (value as f32 / u32::MAX as f32) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm_samples(wav: &[u8]) -> impl Iterator<Item = i16> + '_ {
        wav[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]))
    }

    #[test]
    fn emits_valid_short_mono_pcm_wav_for_every_cue() {
        for cue in ALL {
            let wav = synthesize(cue);
            assert_eq!(&wav[0..4], b"RIFF", "{cue:?}");
            assert_eq!(&wav[8..16], b"WAVEfmt ", "{cue:?}");
            assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1, "{cue:?} PCM");
            assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1, "{cue:?} mono");
            assert_eq!(
                u32::from_le_bytes(wav[24..28].try_into().unwrap()),
                SAMPLE_RATE
            );
            assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16);
            assert_eq!(
                u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
                wav.len() - 44
            );
            assert_eq!(
                u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize + 8,
                wav.len()
            );
            let seconds = (wav.len() - 44) as f32 / (SAMPLE_RATE * 2) as f32;
            assert!(seconds > 0.0 && seconds <= 1.4, "{cue:?}: {seconds}s");
            let samples: Vec<_> = pcm_samples(&wav).collect();
            assert!(
                samples.iter().any(|&sample| sample != 0),
                "{cue:?} is silent"
            );
            assert!(
                samples
                    .iter()
                    .all(|&sample| sample.unsigned_abs() <= 20_315),
                "{cue:?} exceeds gain bound"
            );
            assert!(
                samples[samples.len() - 64..]
                    .iter()
                    .all(|&sample| sample.unsigned_abs() <= 8),
                "{cue:?} tail is not quiet"
            );
        }
    }

    #[test]
    fn cues_are_deterministic_and_distinct() {
        let rendered: Vec<_> = ALL.into_iter().map(synthesize).collect();
        for (index, wav) in rendered.iter().enumerate() {
            assert_eq!(*wav, synthesize(ALL[index]));
            assert!(
                rendered[index + 1..].iter().all(|other| other != wav),
                "duplicate waveform for {:?}",
                ALL[index]
            );
        }
    }
}
