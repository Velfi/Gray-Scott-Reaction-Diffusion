# Gray Scott Reaction Diffusion

Reaction–diffusion systems are mathematical models which correspond to several physical phenomena: the most common is the change in space and time of the concentration of one or more chemical substances: local chemical reactions in which the substances are transformed into each other, and diffusion which causes the substances to spread out over a surface in space. [Wikipedia]

If that sounds like gibberish, just know that it's an algorithm for modeling the processes that create lots of natural patterns.
For more cool patterns in nature, [see here][patterns-in-nature].

![alt text][example_soliton]
![alt text][example_brain_coral]

## Running The Visualizer

You'll need to have Rust and `cargo` installed. Then, run `cargo run --release` in you terminal of choice. Click and drag around to seed the reaction and interact with it.

## Seeing Different Reactions

Right now, changing the default interaction is somewhat inconvenient, but I've provided a few presets that you can manually enter. Just update line 28 in `main.rs`

```rust
// src/main.rs:28
// Change the preset to anything exported from `model_presets`
const CURRENT_MODEL: (f32, f32) = model_presets::SOLITON_COLLAPSE;
```

The options available are:

- `BRAIN_CORAL`
- `FINGERPRINT`
- `MITOSIS`
- `RIPPLES`
- `SOLITON_COLLAPSE`
- `U_SKATE_WORLD`
- `UNDULATING`
- `WORMS`

The simulation runs pretty slowly right now because of the inefficient way I'm calculating it. I hope to improve on this implementation by rewriting things in GLSL someday.

[wikipedia]: https://en.wikipedia.org/wiki/Reaction%E2%80%93diffusion_system
[patterns-in-nature]: https://en.wikipedia.org/wiki/Patterns_in_nature
[example_soliton]: /example_soliton.png "An example of the 'Soliton Collapse' setting"
[example_brain_coral]: /example_brain_coral.png "An example of the 'Brain Coral' setting"
