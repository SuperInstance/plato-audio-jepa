# plato-audio-jepa

Audio / vibration JEPA for the **PLATO nervous system** — processes microphone input into structured 16-dimensional room state vectors.

## Signal Chain

```
Microphone → Audio Samples → AudioDeadband → Audio-1.5B JEPA → RoomAudioState → plato-nervous
```

This crate sits in the **perception layer** of the PLATO nervous system:

1. **Sensor** — raw audio samples arrive from room microphones
2. **Deadband** (`AudioDeadband`) — spectral diff filter: only process when frequency content changes significantly
3. **Audio-1.5B JEPA** — extracts a `RoomAudioState` (16-dim vector covering volume, frequency, bands, temporal patterns)
4. **Room State Vector** — the 16-dim state feeds into the downstream nervous system (`plato-nervous`)

## State Vector Layout (16-dim)

| Index | Field | Description |
|-------|-------|-------------|
| 0 | `volume` | Normalized volume (0–1) |
| 1 | `dominant_frequency` | Dominant freq normalized (0–1) |
| 2 | `spectral_centroid` | Brightness of sound |
| 3 | `anomaly_score` | Anomaly confidence (0–1) |
| 4–7 | `band_energies` | Sub-bass, bass, mid, high |
| 8–11 | `temporal_patterns` | Volume trend, freq drift, onset rate, rhythm |
| 12–15 | `reserved` | Future use |

## Key Types

- **`AudioTile`** — structured audio state (volume, dominant_frequency, spectral_centroid, anomaly)
- **`AudioDeadband`** — spectral deadband; skips unchanged audio windows
- **`RoomAudioState`** — the 16-dim state vector

## Key Functions

- `compute_spectrum(samples) → Vec<f32>` — simple DFT (first 16 bins)
- `compute_spectral_centroid(spectrum) → f32`
- `compute_band_energy(spectrum, low, high) → f32`
- `compute_onset_rate(volume_history) → f32` — detects sudden volume changes
- `detect_rhythm(volume_history, sample_rate) → f32` — estimates BPM
- `audio_state_to_tile(state) → AudioTile`

## Usage

```rust
use plato_audio_jepa::*;

let mut deadband = AudioDeadband::new(0.05);
let spectrum = compute_spectrum(&audio_samples);

if deadband.should_process(&spectrum) {
    let state = RoomAudioState {
        volume: 0.5,
        dominant_frequency: 0.3,
        spectral_centroid: compute_spectral_centroid(&spectrum),
        anomaly_score: 0.1,
        band_energies: [
            compute_band_energy(&spectrum, 0, 4),
            compute_band_energy(&spectrum, 4, 8),
            compute_band_energy(&spectrum, 8, 12),
            compute_band_energy(&spectrum, 12, 16),
        ],
        temporal_patterns: [0.0; 4],
        reserved: [0.0; 4],
    };
    let tile = audio_state_to_tile(&state);
    // feed tile into plato-nervous
}
```

## Ecosystem

plato-audio-jepa is part of the **PLATO Nervous System** — the acoustic perception layer.

**Where this sits:** Layer 0 (sensor input). Produces 16-dimensional audio state vectors that flow into [plato-nervous](https://github.com/SuperInstance/plato-nervous) for RoomStateVector fusion.

**Signal chain:**
```
Camera → plato-vision-jepa (16-dim) ─┐
                                      ├→ plato-nervous (RoomStateVector) → distillation
Microphone → plato-audio-jepa (16-dim)─┘
```

| Repo | Role |
|------|------|
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | Core signal chain — consumes audio state vectors |
| [plato-vision-jepa](https://github.com/SuperInstance/plato-vision-jepa) | Sister crate — 16-dim vision state vectors |
| [openconstruct-kernel](https://github.com/SuperInstance/openconstruct-kernel) | Hardware detection for microphone devices |
| [concrete-token-demo](https://github.com/SuperInstance/concrete-token-demo) | CLI demo that can exercise audio state inputs |
| [plato-browser](https://github.com/SuperInstance/plato-browser) | Browser demo using Web Audio API |
| [luciddreamer-ai](https://github.com/SuperInstance/luciddreamer-ai) | Cloud-layer reactive podcast engine |
| [hermit-crab](https://github.com/SuperInstance/hermit-crab) | Agent migration between rooms |

See [DEPENDENCIES.md](./DEPENDENCIES.md) for detailed dependency and data flow information.

## License

MIT
