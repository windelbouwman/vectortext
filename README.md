
# Vector text

Rendering text in openGL using another approach then textures.

Demo on how to use three rust packages in combination:

- [rusttype](https://crates.io/crates/rusttype): to load ttf font files, and create curves from text using this font
- [lyon](https://crates.io/crates/lyon): to tesselate the resulting contours of the fonts
- [glium](https://crates.io/crates/glium): to display those vertices in openGL

