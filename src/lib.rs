//! # plato-audio-jepa
//!
//! Audio / vibration JEPA for the PLATO nervous system. Processes microphone
//! input into structured room state vectors suitable for downstream tiles.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Structured room state produced by the audio JEPA from a chunk of audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTile {
    pub id: Uuid,
    pub volume: f32,
    pub dominant_frequency: f32,
    pub spectral_centroid: f32,
    pub anomaly: f32,
    pub timestamp: u64,
}

/// Spectral deadband filter — only process chunks when frequency content changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeadband {
    pub threshold: f64,
    pub last_spectrum: Option<Vec<f32>>,
}

impl Default for AudioDeadband {
    fn default() -> Self {
        Self {
            threshold: 0.05,
            last_spectrum: None,
        }
    }
}

impl AudioDeadband {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            last_spectrum: None,
        }
    }

    /// Returns `true` if the new spectrum represents a significant change.
    pub fn should_process(&mut self, spectrum: &[f32]) -> bool {
        let significant = match self.last_spectrum {
            None => true,
            Some(ref prev) => {
                let n = prev.len().min(spectrum.len());
                if n == 0 {
                    return true;
                }
                let diff: f64 = (0..n)
                    .map(|i| {
                        let d = (prev[i] - spectrum[i]) as f64;
                        d * d
                    })
                    .sum::<f64>()
                    / n as f64;
                diff.sqrt() > self.threshold
            }
        };
        if significant {
            self.last_spectrum = Some(spectrum.to_vec());
        }
        significant
    }
}

/// 16-dimensional audio state vector for a room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAudioState {
    /// [0] volume (0-1 normalized)
    pub volume: f32,
    /// [1] dominant_frequency (Hz normalized to 0-1, where 1 = Nyquist)
    pub dominant_frequency: f32,
    /// [2] spectral_centroid (brightness of sound)
    pub spectral_centroid: f32,
    /// [3] anomaly_score (0-1)
    pub anomaly_score: f32,
    /// [4-7] frequency band energies (sub-bass, bass, mid, high)
    pub band_energies: [f32; 4],
    /// [8-11] temporal patterns (volume trend, frequency drift, onset rate, rhythm)
    pub temporal_patterns: [f32; 4],
    /// [12-15] reserved
    pub reserved: [f32; 4],
}

impl Default for RoomAudioState {
    fn default() -> Self {
        Self {
            volume: 0.0,
            dominant_frequency: 0.0,
            spectral_centroid: 0.0,
            anomaly_score: 0.0,
            band_energies: [0.0; 4],
            temporal_patterns: [0.0; 4],
            reserved: [0.0; 4],
        }
    }
}

impl RoomAudioState {
    /// Convert to a flat 16-element f32 array.
    pub fn to_vector(&self) -> [f32; 16] {
        let mut v = [0.0f32; 16];
        v[0] = self.volume;
        v[1] = self.dominant_frequency;
        v[2] = self.spectral_centroid;
        v[3] = self.anomaly_score;
        v[4..8].copy_from_slice(&self.band_energies);
        v[8..12].copy_from_slice(&self.temporal_patterns);
        v[12..16].copy_from_slice(&self.reserved);
        v
    }

    /// Reconstruct from a flat 16-element f32 array.
    pub fn from_vector(v: &[f32; 16]) -> Self {
        let mut bands = [0.0f32; 4];
        let mut temporal = [0.0f32; 4];
        let mut reserved = [0.0f32; 4];
        bands.copy_from_slice(&v[4..8]);
        temporal.copy_from_slice(&v[8..12]);
        reserved.copy_from_slice(&v[12..16]);
        Self {
            volume: v[0],
            dominant_frequency: v[1],
            spectral_centroid: v[2],
            anomaly_score: v[3],
            band_energies: bands,
            temporal_patterns: temporal,
            reserved,
        }
    }
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Compute a simple DFT returning the magnitude of the first 16 frequency bins.
/// Input is a window of time-domain samples (ideally power-of-2 length).
pub fn compute_spectrum(samples: &[f32]) -> Vec<f32> {
    let n_bins = 16usize;
    let n = samples.len().max(1);
    let mut magnitudes = vec![0.0f32; n_bins];

    for (k, mag) in magnitudes.iter_mut().enumerate() {
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let angle = -2.0 * std::f32::consts::PI * (k as f32) * (i as f32) / (n as f32);
            re += s * angle.cos();
            im += s * angle.sin();
        }
        *mag = (re * re + im * im).sqrt() / (n as f32);
    }

    magnitudes
}

/// Compute the spectral centroid (weighted mean of frequency bins).
/// Returns a value in the range of the spectrum magnitudes.
pub fn compute_spectral_centroid(spectrum: &[f32]) -> f32 {
    if spectrum.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0f32;
    let mut weight_total = 0.0f32;

    for (i, &mag) in spectrum.iter().enumerate() {
        let freq_bin = i as f32;
        weighted_sum += freq_bin * mag;
        weight_total += mag;
    }

    if weight_total == 0.0 {
        0.0
    } else {
        weighted_sum / weight_total
    }
}

/// Compute the energy (sum of squares) in a frequency range.
pub fn compute_band_energy(spectrum: &[f32], low: usize, high: usize) -> f32 {
    let low = low.min(spectrum.len());
    let high = high.min(spectrum.len());
    if low >= high {
        return 0.0;
    }
    spectrum[low..high].iter().map(|&m| m * m).sum()
}

/// Estimate the onset rate (sudden volume increases per unit time) from a
/// volume history. Returns the rate of onsets.
pub fn compute_onset_rate(volume_history: &[f32]) -> f32 {
    if volume_history.len() < 2 {
        return 0.0;
    }

    let threshold = 0.1;
    let mut onsets = 0usize;

    for i in 1..volume_history.len() {
        let delta = volume_history[i] - volume_history[i - 1];
        if delta > threshold {
            onsets += 1;
        }
    }

    onsets as f32 / (volume_history.len() - 1) as f32
}

/// Estimate BPM from a volume envelope by counting peaks.
/// `sample_rate` is the rate of the volume-history samples (not audio samples).
pub fn detect_rhythm(volume_history: &[f32], sample_rate: f32) -> f32 {
    if volume_history.len() < 4 || sample_rate <= 0.0 {
        return 0.0;
    }

    // Simple peak detection: a sample is a peak if it's greater than its neighbors
    let mut peaks: Vec<usize> = Vec::new();
    for i in 1..volume_history.len() - 1 {
        if volume_history[i] > volume_history[i - 1]
            && volume_history[i] > volume_history[i + 1]
            && volume_history[i] > 0.05
        {
            peaks.push(i);
        }
    }

    if peaks.len() < 2 {
        return 0.0;
    }

    // Average period between peaks → BPM
    let mut total_interval = 0.0f32;
    let mut count = 0usize;
    for w in peaks.windows(2) {
        let interval_samples = (w[1] - w[0]) as f32;
        let interval_seconds = interval_samples / sample_rate;
        if interval_seconds > 0.1 && interval_seconds < 5.0 {
            total_interval += interval_seconds;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }

    let avg_period = total_interval / count as f32;
    if avg_period > 0.0 {
        60.0 / avg_period
    } else {
        0.0
    }
}

/// Convert a RoomAudioState into an AudioTile (the output artifact).
pub fn audio_state_to_tile(state: &RoomAudioState) -> AudioTile {
    AudioTile {
        id: Uuid::new_v4(),
        volume: state.volume,
        dominant_frequency: state.dominant_frequency,
        spectral_centroid: state.spectral_centroid,
        anomaly: state.anomaly_score,
        timestamp: 0,
    }
}
