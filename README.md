# Gray Scott Reaction Diffusion

Reaction–diffusion systems are mathematical models which correspond to several physical phenomena: the most common is the change in space and time of the concentration of one or more chemical substances: local chemical reactions in which the substances are transformed into each other, and diffusion which causes the substances to spread out over a surface in space. [Wikipedia]

If that sounds like gibberish, just know that it's an algorithm for modeling the processes that create lots of natural patterns.
For more cool patterns in nature, [see here][patterns-in-nature].

![Example 1][example_1]
![Example 2][example_2]
![Example 3][example_3]
![Example 4][example_4]

## Running The Visualizer

You'll need to have Rust and `cargo` installed. Then, run `cargo run --release` in your terminal of choice.

## Controls

- **Left Mouse Button**: Click and drag to seed the reaction
- **Right Mouse Button**: Click and drag to interact with the reaction
- **C**: Clear the screen
- **N**: Fill the screen with noise
- **G**: Cycle through different color gradients (hold SHIFT to cycle backwards)
- **P**: Cycle through different reaction presets (hold SHIFT to cycle backwards)
- **U**: Cycle through different nutrient patterns (hold SHIFT to cycle backwards)
- **? or \\**: Toggle help overlay
- **ESC**: Exit the application

## Reaction Presets

The simulation comes with several built-in presets that create different patterns:

- `BRAIN_CORAL`
- `FINGERPRINT`
- `MITOSIS`
- `RIPPLES`
- `SOLITON_COLLAPSE`
- `U_SKATE_WORLD`
- `UNDULATING`
- `WORMS`

## Nutrient Patterns

The simulation also includes various nutrient patterns that affect how the reaction spreads:

- Uniform
- Checkerboard
- Diagonal Gradient
- Radial Gradient
- Vertical Stripes
- Horizontal Stripes
- Noise

The simulation now runs efficiently using parallel processing with Rayon. The window title shows the current preset name, feed rate (f), kill rate (k), nutrient pattern, and FPS.

[wikipedia]: https://en.wikipedia.org/wiki/Reaction%E2%80%93diffusion_system
[patterns-in-nature]: https://en.wikipedia.org/wiki/Patterns_in_nature
[example_1]: /example_1.png "Example of the Gray-Scott Reaction Diffusion simulation"
[example_2]: /example_2.png "Another example of the Gray-Scott Reaction Diffusion simulation"
