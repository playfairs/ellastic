# Ellastic Features

This document defines the planned feature set for **Ellastic**.

Ellastic is designed as an experimental media manipulation framework centered around the idea that digital media is ultimately data. Instead of treating files as immutable representations, Ellastic allows their underlying bytes, structures, signals, and encoded information to be inspected, transformed, layered, and intentionally corrupted.

The features below are organized from foundational infrastructure to higher-level media manipulation.

---

## 1. Core Architecture

### 1.1 Media Abstraction Layer

Ellastic should represent media through a common internal abstraction rather than having every operation directly manipulate files.

A media object should be able to expose:

* Raw byte data
* File size
* MIME type
* File extension
* Detected format
* Metadata
* Binary regions
* Logical streams
* Channels
* Frames
* Samples
* Dimensions
* Duration
* Bit depth
* Sample rate
* Color information

Operations should be able to operate on this abstraction without needing to know how the original file was loaded.

### 1.2 Format Detection

Automatically identify supported media formats.

Detection should use:

* File signatures / magic bytes
* Container headers
* MIME types
* File extensions as a fallback

The detector should distinguish between:

* Images
* Audio
* Video
* Containers
* Raw binary data
* Unknown formats

Unknown files should still be usable through the raw binary manipulation system.

### 1.3 Raw Binary Mode

Every file should be capable of being opened as raw data.

Provide access to:

* Byte offsets
* Hexadecimal representation
* Binary representation
* Decimal values
* Byte ranges
* File size
* Entropy information

Raw mode should not require the file to be recognized as a valid media format.

### 1.4 Non-Destructive Processing

Operations should never modify the original file unless explicitly requested.

The default workflow should be:

```text
Input
  ↓
Operation
  ↓
New Output
```

Users should be able to build transformation chains without repeatedly overwriting source material.

### 1.5 Deterministic Operations

Operations involving randomness should support explicit seeds.

Example:

```text
seed = 4787435874
```

Running the same operation with the same input and seed should produce the same result.

This allows corrupted media to be reproduced exactly.

---

# 2. Transformation Pipeline

The pipeline is the core of Ellastic.

Users should be able to combine multiple operations into a layered transformation sequence.

## 2.1 Operation Chains

Support chains such as:

```text
Load
→ Slice
→ Byte Shift
→ XOR
→ Bit Manipulation
→ Compression
→ Reconstruct
→ Export
```

Each operation should receive the output of the previous operation.

## 2.2 Layered Operations

Operations should be stackable.

For example:

```text
Image
 ↓
Channel Split
 ↓
RGB Shift
 ↓
Bit Flip
 ↓
Byte Shuffle
 ↓
Scanline Offset
 ↓
JPEG Re-encode
```

Each layer should be independently configurable.

## 2.3 Operation Parameters

Every operation should expose structured parameters.

Example:

```text
xor
  key: 0xFF
  offset: 1024
  length: 8192
```

Parameters should be serializable so that transformations can be saved and reproduced.

## 2.4 Pipeline Serialization

Allow transformation pipelines to be saved to a file.

A pipeline should contain:

* Input configuration
* Operations
* Parameters
* Random seeds
* Output configuration

The pipeline should be executable again without manually recreating every operation.

## 2.5 Pipeline Preview

Provide a way to preview the result of a pipeline before exporting.

Preview should support:

* Images
* Audio
* Video
* Raw binary changes

For expensive operations, previewing a reduced representation should be possible.

---

# 3. Databending

Databending is one of Ellastic's primary purposes.

## 3.1 Byte Mutation

Implement controlled modification of arbitrary byte ranges.

Operations should include:

* Replace
* Add
* Remove
* Duplicate
* Swap
* Reverse
* Rotate
* Shuffle
* Repeat
* Truncate
* Expand

Example:

```text
bytes 0x1000-0x2000
→ reverse
```

## 3.2 Byte Arithmetic

Perform arithmetic directly on bytes.

Operations:

* Add
* Subtract
* Multiply
* Divide
* Modulo
* Minimum
* Maximum

Support:

* Constant values
* Per-byte values
* Repeating patterns
* Random values

## 3.3 Bit Manipulation

Provide individual bit-level operations.

Features:

* Bit flip
* Bit clear
* Bit set
* Bit rotation
* Bit reversal
* Bit masking
* AND
* OR
* XOR
* NOT

Allow operations to target:

* Individual bytes
* Byte ranges
* Bit positions
* Periodic offsets

## 3.4 XOR Operations

Provide multiple XOR modes:

* Constant XOR
* Repeating-key XOR
* Random XOR
* Position-dependent XOR
* XOR against another file
* XOR against generated data

Example:

```text
input XOR pattern
```

## 3.5 Byte Swapping

Support:

* Adjacent byte swapping
* Endianness reversal
* Word swapping
* Block swapping
* Randomized swapping

Configurable block sizes should be supported.

## 3.6 Byte Rotation

Rotate bytes within:

* Entire files
* Fixed-size blocks
* Selected ranges
* Repeating windows

Both left and right rotation should be supported.

---

# 4. Corruption Engine

Ellastic should provide deliberate corruption operations that operate at different levels.

## 4.1 Random Corruption

Randomly modify selected regions.

Parameters:

* Corruption probability
* Corruption strength
* Region size
* Seed
* Distribution

## 4.2 Burst Corruption

Instead of randomly changing individual bytes, corrupt contiguous regions.

Example:

```text
10 KB file
     ↓
corrupt bytes 4200-4700
```

## 4.3 Sparse Corruption

Modify isolated positions across the file.

Useful for subtle corruption.

## 4.4 Progressive Corruption

Gradually increase corruption strength across the file.

Example:

```text
Beginning → 0% corruption
Middle    → 25%
End       → 100%
```

## 4.5 Region-Based Corruption

Divide data into regions and apply different operations to each region.

Example:

```text
Region 1 → XOR
Region 2 → Reverse
Region 3 → Noise
Region 4 → Bit flips
```

## 4.6 Corruption Masks

Allow users to define where corruption occurs using masks.

Masks may be:

* Binary
* Grayscale
* Procedural
* Random
* Generated from another media file

---

# 5. Signal Manipulation

Ellastic should treat audio and other numerical media as signals.

## 5.1 Signal Extraction

Convert supported media into numerical signal representations.

Support:

* Mono
* Stereo
* Multi-channel
* Sample arrays
* Normalized floating-point data

## 5.2 Amplitude Manipulation

Operations:

* Gain
* Attenuation
* Clipping
* Normalization
* Inversion
* Quantization
* Dynamic range reduction

## 5.3 Signal Mathematics

Support mathematical operations on signals.

Examples:

```text
signal + signal
signal - signal
signal × signal
signal ÷ signal
```

Also support:

* Sine
* Cosine
* Tangent
* Absolute value
* Power
* Logarithm
* Exponential

## 5.4 Oscillation

Generate periodic modulation signals.

Parameters:

* Frequency
* Amplitude
* Phase
* Waveform
* Offset

Waveforms should include:

* Sine
* Square
* Triangle
* Sawtooth
* Noise

## 5.5 Modulation

Implement:

* Amplitude modulation
* Frequency modulation
* Ring modulation
* Phase modulation

## 5.6 Signal Folding

Implement wavefolding and numerical folding operations.

This should intentionally push signals beyond their normal range and fold them back into valid ranges.

## 5.7 Sample Manipulation

Allow:

* Sample reversal
* Sample duplication
* Sample deletion
* Sample interpolation
* Sample quantization
* Sample swapping
* Sample reordering

---

# 6. Audio Manipulation

## 6.1 Audio Import

Support common audio formats through a decoding layer.

Initial targets:

* WAV
* AIFF
* FLAC
* OGG
* MP3

## 6.2 Audio Export

Support configurable:

* Sample rate
* Bit depth
* Channel count
* Encoding
* Compression

## 6.3 Channel Operations

Allow:

* Channel splitting
* Channel swapping
* Channel duplication
* Channel deletion
* Channel mixing
* Channel inversion

Example:

```text
Left  → Right
Right → Left
```

## 6.4 Channel Corruption

Apply different transformations to individual channels.

Example:

```text
Left  → normal
Right → XOR corruption
```

## 6.5 Sample Rate Manipulation

Allow intentional sample-rate abuse.

Features:

* Rate increase
* Rate reduction
* Arbitrary resampling
* Sample dropping
* Sample duplication

## 6.6 Bit Depth Manipulation

Allow conversion between bit depths and intentionally invalid or unusual quantization behavior.

---

# 7. Image Manipulation

## 7.1 Image Import

Initial supported formats:

* PNG
* JPEG
* BMP
* TIFF
* WebP
* GIF

## 7.2 Pixel-Level Operations

Support:

* Pixel replacement
* Pixel swapping
* Pixel duplication
* Pixel deletion
* Channel modification
* Random pixel corruption

## 7.3 Channel Manipulation

Support:

* RGB split
* RGBA split
* Channel swapping
* Channel duplication
* Channel inversion
* Channel removal
* Channel arithmetic

Example:

```text
R ← G
G ← B
B ← R
```

## 7.4 Color Mathematics

Implement:

* Additive color modification
* Subtractive modification
* Multiplication
* Division
* XOR
* Bit masking
* Quantization
* Thresholding

## 7.5 Pixel Sorting

Implement configurable pixel sorting.

Sorting parameters should include:

* Direction
* Threshold
* Channel
* Brightness
* Saturation
* Hue
* Sorting function
* Region

## 7.6 Image Slicing

Allow images to be divided into:

* Rows
* Columns
* Rectangles
* Arbitrary regions

Slices should be individually transformable.

## 7.7 Scanline Manipulation

Support:

* Scanline shifting
* Scanline duplication
* Scanline deletion
* Scanline reversal
* Scanline swapping
* Scanline corruption

---

# 8. Image Databending

Image databending should support both decoded pixel manipulation and direct file-level manipulation.

## 8.1 Header Preservation

Allow users to preserve image headers while modifying image payload data.

This should make it possible to intentionally corrupt encoded image data without immediately destroying the entire container.

## 8.2 Payload Corruption

Target specific regions such as:

```text
Header
Metadata
Image payload
Trailer
```

## 8.3 Compression-Aware Corruption

For formats such as JPEG, provide operations that target compressed data rather than decoded pixels.

## 8.4 Re-Encode Loops

Allow:

```text
Decode
→ Modify
→ Encode
→ Decode
→ Modify
→ Encode
```

for repeated generational corruption.

---

# 9. Video Datamoshing

Video should receive a dedicated processing system rather than being treated as a giant image sequence.

## 9.1 Video Stream Detection

Identify:

* Video streams
* Audio streams
* Subtitle streams
* Container metadata

## 9.2 Frame Extraction

Allow operations on individual frames or ranges.

Support:

* Frame selection
* Frame duplication
* Frame deletion
* Frame reversal
* Frame reordering

## 9.3 Keyframe Manipulation

Provide access to:

* I-frames
* P-frames
* B-frames

Operations should be able to target individual frame types.

## 9.4 Frame Duplication

Duplicate selected frames or frame ranges.

## 9.5 Frame Dropping

Remove selected frames.

Support:

* Random dropping
* Periodic dropping
* Range dropping
* Probability-based dropping

## 9.6 Motion Data Manipulation

Where supported by the codec, expose motion-related information for experimental manipulation.

## 9.7 Codec-Aware Datamoshing

Provide codec-specific operations rather than pretending every video codec behaves identically.

Initial target codecs should be selected based on feasibility and available decoding libraries.

---

# 10. Container Manipulation

Ellastic should understand media containers independently from the media streams inside them.

## 10.1 Container Inspection

Display:

* Tracks
* Streams
* Metadata
* Duration
* Codec information
* Bitrate
* Packet structure

## 10.2 Stream Extraction

Extract individual streams from containers.

Example:

```text
video.mp4
 ├── video stream
 ├── audio stream
 └── metadata
```

## 10.3 Stream Replacement

Replace individual streams while preserving the surrounding container when possible.

## 10.4 Metadata Manipulation

Support:

* Metadata viewing
* Metadata removal
* Metadata replacement
* Metadata corruption
* Metadata generation

---

# 11. Procedural Data Generation

Ellastic should be able to generate data instead of only modifying existing data.

## 11.1 Noise Generation

Generate:

* White noise
* Pink noise
* Brown noise
* Binary noise
* Random bytes
* Structured noise

## 11.2 Mathematical Data

Generate data using:

* Sine waves
* Cosine waves
* Fractals
* Noise functions
* Mathematical sequences
* Iterative functions

## 11.3 Byte Patterns

Generate repeating patterns such as:

```text
00 FF 00 FF
AA 55 AA 55
00 01 02 03
```

## 11.4 Procedural Corruption

Use generated functions to determine how corruption changes over space or time.

Example:

```text
corruption(x) = sin(x * frequency)
```

---

# 12. Analysis Tools

Ellastic should not only destroy data. It should also explain what happened to it.

## 12.1 Hex Viewer

Provide a built-in hexadecimal viewer.

Display:

* Offset
* Hexadecimal bytes
* ASCII representation
* Binary representation

## 12.2 Binary Diff

Compare two files byte-by-byte.

Display:

* Changed bytes
* Added bytes
* Removed bytes
* Change percentage
* Changed regions

## 12.3 Entropy Analysis

Calculate entropy across:

* Entire files
* Fixed windows
* Selected ranges
* Media streams

Provide a visualization of entropy changes.

## 12.4 Histogram Analysis

For applicable data, display distributions of:

* Byte values
* Pixel values
* Color channels
* Audio amplitudes

## 12.5 Signal Visualization

Display:

* Waveforms
* Frequency spectra
* Spectrograms
* Channel levels

## 12.6 Image Difference

Compare original and transformed images.

Provide:

* Absolute difference
* Signed difference
* Channel difference
* Heatmap representation

---

# 13. Region Selection

Operations should not always affect the entire file.

## 13.1 Byte Ranges

Allow explicit ranges:

```text
0x1000-0x2000
```

## 13.2 Percentage Ranges

Allow:

```text
0%-25%
50%-75%
```

## 13.3 Random Regions

Select random regions using a deterministic seed.

## 13.4 Repeating Regions

Apply operations every N bytes, samples, pixels, or frames.

## 13.5 Conditional Regions

Allow operations to target data matching conditions.

Examples:

```text
byte == 0xFF
pixel brightness > 0.8
sample amplitude < -0.5
```

---

# 14. Randomization System

Randomness should be a first-class feature.

## 14.1 Seeded Randomness

Every random operation should accept a seed.

## 14.2 Random Distributions

Support:

* Uniform
* Gaussian
* Exponential
* Poisson
* Custom distributions

where applicable.

## 14.3 Random Operation Selection

Select operations randomly from a user-defined set.

Example:

```text
50% XOR
25% reverse
15% bit flip
10% byte shuffle
```

## 14.4 Random Parameter Generation

Generate operation parameters automatically within user-defined bounds.

---

# 15. Reversible Operations

Some operations should support reversal.

Examples:

* XOR
* Byte rotation
* Byte reversal
* Bit inversion
* Channel swapping
* Certain permutations

Ellastic should expose whether an operation is:

```text
Reversible
Partially reversible
Irreversible
```

This information should be available to the pipeline system.

---

# 16. Operation Metadata

Every operation should define metadata describing its behavior.

Example:

```text
Operation:
    Name: XOR
    Category: Binary
    Deterministic: Yes
    Reversible: Yes
    Lossy: No
    Media Types:
        Binary
        Image
        Audio
        Video
```

This allows the UI and pipeline system to automatically determine what operations are valid.

---

# 17. Plugin System

Ellastic should eventually support third-party operations.

## 17.1 Plugin API

Plugins should be able to provide:

* New transformations
* New codecs
* New analysis tools
* New generators
* New file formats

## 17.2 Plugin Metadata

Plugins should declare:

* Name
* Version
* Author
* License
* Supported media types
* Required Ellastic version

## 17.3 Native Plugins

Native plugins may eventually be supported through a stable ABI.

The ABI should remain intentionally small to reduce compatibility problems.

---

# 18. CLI

Ellastic should provide a command-line interface suitable for scripting.

## 18.1 Basic Usage

Example:

```bash
ellastic input.png output.png --operation xor --key ff
```

## 18.2 Pipeline Execution

Example:

```bash
ellastic input.png output.png \
  --pipeline pipeline.ellastic
```

## 18.3 Inspection

Example:

```bash
ellastic inspect input.png
```

## 18.4 Diff

Example:

```bash
ellastic diff original.png corrupted.png
```

## 18.5 Batch Processing

Allow entire directories to be processed.

Example:

```bash
ellastic batch ./input ./output --operation corrupt
```

## 18.6 Scriptable Output

Support machine-readable output such as:

* JSON
* JSON Lines
* Plain text

---

# 19. Interactive Interface

A graphical interface should eventually provide an interactive way to construct transformations.

## 19.1 Pipeline Editor

Represent operations visually:

```text
[Input]
   ↓
[XOR]
   ↓
[Pixel Sort]
   ↓
[Byte Shuffle]
   ↓
[Output]
```

## 19.2 Live Preview

Changes to parameters should update a preview when practical.

## 19.3 Operation Browser

Operations should be organized into categories:

* Binary
* Image
* Audio
* Video
* Signal
* Corruption
* Analysis
* Generation

## 19.4 Parameter Editor

Every operation should expose its parameters through a consistent interface.

## 19.5 Before/After Comparison

Allow side-by-side or overlay comparison.

---

# 20. Presets

Users should be able to save common transformation configurations.

Example presets:

```text
Heavy JPEG Corruption
Audio Bitcrush
RGB Channel Collapse
Random Byte Damage
 VHS-style Datamosh
Progressive Corruption
```

Presets should simply serialize existing operations rather than implementing special-case logic.

---

# 21. Batch and Automation

## 21.1 Directory Processing

Process multiple files using one pipeline.

## 21.2 Recursive Processing

Process directory trees recursively.

## 21.3 Filename Templates

Support generated output names.

Example:

```text
{filename}.ellastic.{ext}
```

## 21.4 Parallel Processing

Independent files should be processable in parallel.

Users should be able to control the worker count.

## 21.5 Watch Mode

Monitor a directory and automatically process newly created files.

Example:

```text
Input directory
     ↓
new file detected
     ↓
pipeline executed
     ↓
output directory
```

---

# 22. Error Handling

Corruption software has a slightly unusual relationship with failure: sometimes the failure is the artwork. The software still needs to know the difference.

## 22.1 Invalid Media Detection

Detect when an operation produces invalid media.

## 22.2 Partial Recovery

Attempt to preserve recoverable portions of damaged files.

## 22.3 Graceful Failure

A failed operation should provide useful information rather than simply crashing.

## 22.4 Corruption Levels

Classify output states:

```text
Valid
Possibly Damaged
Malformed
Unreadable
```

## 22.5 Salvage Mode

Attempt to extract usable streams or data from damaged containers.

---

# 23. Performance

## 23.1 Streaming Processing

Large files should not need to be completely loaded into memory.

Operations should support streaming where possible.

## 23.2 Memory-Mapped Files

Use memory mapping for suitable binary operations.

## 23.3 Parallel Operations

Operations that can safely run independently should support parallel execution.

## 23.4 Incremental Processing

Avoid recomputing the entire pipeline when only a small region has changed.

## 23.5 Preview Resolution

Image and video previews should support reduced resolutions to make experimentation fast.

---

# 24. Reproducibility

Every generated result should be reproducible whenever possible.

## 24.1 Transformation Logs

Store:

* Input file hash
* Output file hash
* Ellastic version
* Operation list
* Parameters
* Random seeds
* Format information

## 24.2 Embedded Provenance

Optionally embed Ellastic metadata into supported output formats.

## 24.3 Pipeline Hashes

Generate a unique hash representing the transformation configuration.

Example:

```text
pipeline: 8e3f...
```

This makes it possible to identify exactly which transformation produced an output.

---

# 25. Safety and File Protection

Ellastic is intentionally capable of producing broken files, so basic guardrails are useful.

## 25.1 Original Preservation

Never overwrite the source by default.

## 25.2 Output Validation

Validate output where possible before replacing an existing output.

## 25.3 Dry Run

Allow users to inspect what an operation would affect without writing changes.

Example:

```bash
ellastic input.bin --operation corrupt --dry-run
```

## 25.4 Maximum File Size

Allow users to configure processing limits.

## 25.5 Maximum Corruption Range

Allow users to restrict operations to defined regions.

---

# 26. Format Support Roadmap

Format support should be implemented progressively.

## Phase 1

* Raw binary
* PNG
* WAV
* BMP

## Phase 2

* JPEG
* TIFF
* FLAC
* AIFF
* WebP

## Phase 3

* MP3
* OGG
* MP4
* MKV
* WebM

## Phase 4

* Additional codecs
* Additional containers
* Plugin-provided formats

Format support should not be considered complete merely because a file can be opened. Ellastic should eventually support meaningful manipulation of the format's internal structure.

---

# 27. Development Phases

## Phase 0: Foundation

* [ ] Project architecture
* [ ] Media abstraction
* [ ] File loading
* [ ] File writing
* [ ] Format detection
* [ ] Raw binary representation
* [ ] Operation interface
* [ ] Pipeline interface
* [ ] Error handling
* [ ] Test infrastructure

## Phase 1: Binary Manipulation

* [ ] Byte replacement
* [ ] Byte insertion
* [ ] Byte deletion
* [ ] Byte duplication
* [ ] Byte reversal
* [ ] Byte rotation
* [ ] Byte swapping
* [ ] XOR
* [ ] AND
* [ ] OR
* [ ] NOT
* [ ] Bit flipping
* [ ] Bit rotation
* [ ] Byte arithmetic
* [ ] Random corruption
* [ ] Seeded randomness

## Phase 2: Image Manipulation

* [ ] PNG decoding
* [ ] PNG encoding
* [ ] BMP decoding
* [ ] BMP encoding
* [ ] Pixel manipulation
* [ ] Channel manipulation
* [ ] Image slicing
* [ ] Scanline manipulation
* [ ] Image corruption
* [ ] Image diff
* [ ] Histogram analysis

## Phase 3: Audio and Signals

* [ ] WAV decoding
* [ ] WAV encoding
* [ ] Sample manipulation
* [ ] Channel manipulation
* [ ] Amplitude operations
* [ ] Signal mathematics
* [ ] Oscillators
* [ ] Noise generation
* [ ] Modulation
* [ ] Waveform visualization
* [ ] Spectrogram generation

## Phase 4: Advanced Databending

* [ ] JPEG payload manipulation
* [ ] Compression-aware corruption
* [ ] Re-encoding loops
* [ ] Metadata manipulation
* [ ] Container inspection
* [ ] Stream extraction
* [ ] Stream replacement

## Phase 5: Video

* [ ] Video stream detection
* [ ] Frame extraction
* [ ] Frame manipulation
* [ ] Frame duplication
* [ ] Frame dropping
* [ ] Keyframe detection
* [ ] Codec-aware operations
* [ ] Datamoshing pipeline

## Phase 6: Automation

* [ ] Pipeline serialization
* [ ] Presets
* [ ] Batch processing
* [ ] Recursive processing
* [ ] Watch mode
* [ ] Parallel processing
* [ ] Machine-readable output
* [ ] Reproducibility metadata

## Phase 7: Interface

* [ ] Interactive pipeline editor
* [ ] Operation browser
* [ ] Parameter editor
* [ ] Live previews
* [ ] Hex viewer
* [ ] Waveform viewer
* [ ] Image comparison
* [ ] Pipeline visualization

## Phase 8: Extensibility

* [ ] Plugin API
* [ ] Plugin discovery
* [ ] Plugin metadata
* [ ] Stable plugin ABI
* [ ] Custom operations
* [ ] Custom format providers
* [ ] Custom analysis providers

---

# 28. Design Principles

Ellastic should follow several principles throughout development.

### Data First

Media should be treated as data before it is treated as a particular file format.

### Layer Everything

Operations should be composable rather than being isolated features.

### Preserve the Original

Destructive experimentation should be explicit.

### Make Randomness Reproducible

Random corruption should still be repeatable.

### Fail Gracefully

Invalid media is an expected outcome. Application crashes are not.

### Expose the Internals

Ellastic should make it possible to inspect what an operation actually changed.

### Prefer Composition Over Special Cases

A complex effect should ideally be achievable by combining primitive operations rather than implementing every visual effect as its own hard-coded feature.

### Support Controlled Destruction

Corruption should be configurable, measurable, and reproducible.

### Unknown Data Is Still Data

A file does not need to be recognized as a supported media format to be manipulated.

---

# 29. Long-Term Goal

The long-term goal of Ellastic is to become a general-purpose experimental framework for manipulating digital media at multiple layers.

A single workflow should eventually be able to move between:

```text
File
 ↓
Container
 ↓
Stream
 ↓
Encoded Data
 ↓
Decoded Data
 ↓
Signal
 ↓
Pixels / Samples
 ↓
Mathematical Representation
```

and back again.

The interesting part of databending is not simply making a file look broken.

It is exploring what happens when the assumptions made by digital media formats are deliberately violated, transformed, recombined, and reconstructed.

Ellastic should provide the tools necessary to experiment with those boundaries while keeping the process controllable, inspectable, and reproducible.

> **Break the data. Observe the result. Layer the transformation. Repeat.**

