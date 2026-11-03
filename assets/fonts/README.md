# Interface font

`Oxanium.ttf` is the original variable font, licensed under `OFL-Oxanium.txt`.
Its default weight is 200. The renderer embeds `Oxanium-Medium.ttf`, a static
weight-500 instance, for readable text across every theme and picker preview.
Both native and WASM builds use the same font.

To regenerate the static instance with FontTools:

```sh
fonttools varLib.instancer assets/fonts/Oxanium.ttf wght=500 --output assets/fonts/Oxanium-Medium.ttf
```
