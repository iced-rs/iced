// The nudge applied when snapping coordinates to the pixel grid.
//
// The CPU applies the identical nudge (see `src/nudge.rs`) when
// deriving clip (scissor) bounds, so that they align exactly with the
// snapped edges of the quads drawn by the vertex shader. Keep the two in
// sync!
const nudge: vec2<f32> = vec2(0.001, 0.001);
