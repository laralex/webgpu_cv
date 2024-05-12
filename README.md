# Web CV of Alexey Larionov
WebGPU + Rust + JS (VanJS) + CI

Known issues:
- Demos overall:
   - fullscreen with F11 is weird (js error in firefox)
   - dynamic resolution confgurable per demo / pixel budget of the screen
   - simple draw abstractions:
      - blit texture
      - axes
      - lines
      - dots
      - text from premade atlas
   - demo panic - progress bar gets stuck
   - debug render mode (maybe visualize demo state in JS)
      - frame idx
      - fps
      - avg fps
      - triangle count
      - vertex count
      - resolution
- Demos:
   - frame prediction
   - fractal with high precision (perturbation)
   - water
   - deferred rendering
   - webXR (VR fractal)
   - gaussian splatting
   - animated avatar (neural net)
   - procedural generation
- Mobile layout
- Sometimes English font is not loaded
- Demo loading until minimal renderable state is loaded, then use demo to render every frame and continue to load
- CV localization (Korean, French?)
- Choose wgpu backend from JS, check backend in wasm