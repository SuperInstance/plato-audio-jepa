# DEPENDENCIES — plato-audio-jepa

## Signal Chain Layer

**L0 (Sensor Input) — Acoustic Room Perception**

Acoustic room perception crate. Produces 16-dimensional audio state vectors from spectrum analysis, rhythm detection, and acoustic environment modeling.

## Ecosystem Dependencies

| Repo | Relationship | Description |
|------|-------------|-------------|
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | **Depended on by** | Consumes audio state vectors for RoomStateVector fusion and the signal chain |
| [openconstruct-kernel](https://github.com/SuperInstance/openconstruct-kernel) | **Related** | May provide raw microphone hardware detection and audio device metadata |
| [concrete-token-demo](https://github.com/SuperInstance/concrete-token-demo) | **Related** | Can be exercised through the concrete-token-demo CLI |

## Data Flow

```
IN:
  - Audio samples (PCM, microphone input)
  - Audio device metadata (sample rate, channels)

OUT:
  - 16-dim audio state vector (A₀..A₁₅)
  - Spectrum analysis summary
  - Rhythm/bPM detection
  - Acoustic environment classification
```
