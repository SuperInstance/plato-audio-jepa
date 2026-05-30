use plato_audio_jepa::*;

#[test]
fn test_compute_spectrum_dc_signal() {
    // DC signal (constant) should have all energy in bin 0
    let samples = vec![1.0f32; 64];
    let spectrum = compute_spectrum(&samples);
    assert_eq!(spectrum.len(), 16);
    assert!(spectrum[0] > 0.9, "DC bin should be dominant, got {}", spectrum[0]);
    for k in 1..16 {
        assert!(spectrum[k] < 0.01, "higher bins should be near 0, bin {k} = {}", spectrum[k]);
    }
}

#[test]
fn test_compute_spectrum_empty() {
    let spectrum = compute_spectrum(&[]);
    assert_eq!(spectrum.len(), 16);
    assert!(spectrum.iter().all(|&m| m < 1e-6));
}

#[test]
fn test_compute_spectral_centroid_single_bin() {
    let spectrum = vec![0.0, 1.0, 0.0, 0.0];
    let centroid = compute_spectral_centroid(&spectrum);
    assert!((centroid - 1.0).abs() < 1e-6, "centroid should be 1.0, got {centroid}");
}

#[test]
fn test_compute_spectral_centroid_empty() {
    assert_eq!(compute_spectral_centroid(&[]), 0.0);
}

#[test]
fn test_compute_spectral_centroid_flat() {
    let spectrum = vec![1.0; 8];
    let centroid = compute_spectral_centroid(&spectrum);
    let expected = (0.0 + 1.0 + 2.0 + 3.0 + 4.0 + 5.0 + 6.0 + 7.0) / 8.0;
    assert!((centroid - expected).abs() < 1e-6, "expected {expected}, got {centroid}");
}

#[test]
fn test_compute_band_energy_basic() {
    let spectrum = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let energy = compute_band_energy(&spectrum, 1, 4);
    let expected = 4.0 + 9.0 + 16.0; // 2^2 + 3^2 + 4^2
    assert!((energy - expected).abs() < 1e-6, "expected {expected}, got {energy}");
}

#[test]
fn test_compute_band_energy_out_of_range() {
    let spectrum = vec![1.0, 2.0];
    let energy = compute_band_energy(&spectrum, 5, 10);
    assert_eq!(energy, 0.0);
}

#[test]
fn test_compute_onset_rate_empty() {
    assert_eq!(compute_onset_rate(&[]), 0.0);
    assert_eq!(compute_onset_rate(&[0.5]), 0.0);
}

#[test]
fn test_compute_onset_rate_no_onsets() {
    let history = vec![0.1, 0.1, 0.1, 0.1];
    let rate = compute_onset_rate(&history);
    assert_eq!(rate, 0.0);
}

#[test]
fn test_compute_onset_rate_with_onsets() {
    let history = vec![0.0, 0.5, 0.0, 0.6, 0.0, 0.5];
    let rate = compute_onset_rate(&history);
    // Three onsets out of 5 intervals
    let expected = 3.0 / 5.0;
    assert!((rate - expected).abs() < 1e-6, "expected {expected}, got {rate}");
}

#[test]
fn test_detect_rhythm_empty() {
    assert_eq!(detect_rhythm(&[], 10.0), 0.0);
    assert_eq!(detect_rhythm(&[0.5], 10.0), 0.0);
}

#[test]
fn test_detect_rhythm_simple_beat() {
    // Simulate a volume envelope at ~2 Hz (120 BPM) with sample_rate=10
    let mut history = vec![0.0f32; 40];
    // Peaks every 5 samples
    for i in 0..8 {
        history[i * 5] = 0.8;
    }
    let bpm = detect_rhythm(&history, 10.0);
    // Period = 5 samples / 10 samples_per_sec = 0.5 sec → 120 BPM
    assert!(bpm > 100.0 && bpm < 140.0, "expected ~120 BPM, got {bpm}");
}

#[test]
fn test_audio_state_roundtrip() {
    let state = RoomAudioState {
        volume: 0.5,
        dominant_frequency: 0.3,
        spectral_centroid: 4.5,
        anomaly_score: 0.1,
        band_energies: [0.1, 0.2, 0.3, 0.4],
        temporal_patterns: [0.5, 0.6, 0.7, 0.8],
        reserved: [0.0; 4],
    };
    let v = state.to_vector();
    let restored = RoomAudioState::from_vector(&v);
    assert!((restored.volume - 0.5).abs() < 1e-6);
    assert!((restored.dominant_frequency - 0.3).abs() < 1e-6);
    assert!((restored.spectral_centroid - 4.5).abs() < 1e-6);
    assert_eq!(restored.band_energies, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(restored.temporal_patterns, [0.5, 0.6, 0.7, 0.8]);
}

#[test]
fn test_audio_state_to_tile() {
    let state = RoomAudioState {
        volume: 0.7,
        dominant_frequency: 0.4,
        spectral_centroid: 3.0,
        anomaly_score: 0.2,
        band_energies: [0.0; 4],
        temporal_patterns: [0.0; 4],
        reserved: [0.0; 4],
    };
    let tile = audio_state_to_tile(&state);
    assert!((tile.volume - 0.7).abs() < 1e-6);
    assert!((tile.dominant_frequency - 0.4).abs() < 1e-6);
    assert!((tile.spectral_centroid - 3.0).abs() < 1e-6);
    assert!((tile.anomaly - 0.2).abs() < 1e-6);
}

#[test]
fn test_audio_deadband_first_spectrum() {
    let mut db = AudioDeadband::new(0.05);
    let s = vec![0.5; 16];
    assert!(db.should_process(&s));
}

#[test]
fn test_audio_deadband_no_change() {
    let mut db = AudioDeadband::new(0.05);
    let s = vec![0.5; 16];
    assert!(db.should_process(&s));
    assert!(!db.should_process(&s), "identical spectrum should not reprocess");
}

#[test]
fn test_audio_deadband_with_change() {
    let mut db = AudioDeadband::new(0.05);
    let s1 = vec![0.5; 16];
    let s2 = vec![1.0; 16];
    assert!(db.should_process(&s1));
    assert!(db.should_process(&s2), "changed spectrum should process");
}
