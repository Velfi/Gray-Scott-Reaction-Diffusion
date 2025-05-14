# Gray Scott Reaction Diffusion

Reaction–diffusion systems are mathematical models which correspond to several physical phenomena: the most common is the change in space and time of the concentration of one or more chemical substances: local chemical reactions in which the substances are transformed into each other, and diffusion which causes the substances to spread out over a surface in space. [Wikipedia]

If that sounds like gibberish, just know that it's an algorithm for modeling the processes that create lots of natural patterns.
For more cool patterns in nature, [see here][patterns-in-nature].

![Example 1][example_1]
![Example 2][example_2]
![Example 3][example_3]
![Example 4][example_4]
![Example 5][example_5]

## Running The Visualizer

You'll need to have Rust and `cargo` installed. Then, run `cargo run --release` in your terminal of choice.

## Controls

- **Left Mouse Button**: Click and drag to seed the reaction
- **Right Mouse Button**: Click and drag to erase/create voids in the reaction
- **Z**: Toggle psychedelic mode (randomly cycles through LUTs)
- **X**: Clear the screen
- **N**: Fill the screen with noise
- **G**: Cycle through different color gradients (hold SHIFT to cycle backwards)
- **P**: Cycle through different reaction presets (hold SHIFT to cycle backwards)
- **U**: Cycle through different nutrient patterns (hold SHIFT to cycle backwards)
- **Arrow Keys**: Adjust feed rate (left/right) and kill rate (up/down) in Custom preset (hold SHIFT for finer control)
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
- `CUSTOM` (Interactive: use arrow keys to adjust feed and kill rates, hold SHIFT for finer control)

## Nutrient Patterns

The simulation also includes various nutrient patterns that affect how the reaction spreads:

- Uniform
- Checkerboard
- Diagonal Gradient
- Radial Gradient
- Vertical Stripes
- Horizontal Stripes
- Noise

[wikipedia]: https://en.wikipedia.org/wiki/Reaction%E2%80%93diffusion_system
[patterns-in-nature]: https://en.wikipedia.org/wiki/Patterns_in_nature
[example_1]: /example_1.png "Example of the Gray-Scott Reaction Diffusion simulation"
[example_2]: /example_2.png "Another example of the Gray-Scott Reaction Diffusion simulation"
[example_3]: /example_3.png "Third example of the Gray-Scott Reaction Diffusion simulation"
[example_4]: /example_4.png "Fourth example of the Gray-Scott Reaction Diffusion simulation"
[example_5]: /example_5.png "Fifth example of the Gray-Scott Reaction Diffusion simulation"
