#version 300 es

out vec4 color;

void main(void) {
   vec4 NDC_COORDS[6];
   NDC_COORDS[0] = vec4(-0.5, -0.5, 0.0, 1.0);
   NDC_COORDS[1] = vec4( 0.0,  0.5, 0.0, 1.0);
   NDC_COORDS[2] = vec4( 0.5, -0.5, 0.0, 1.0);
   NDC_COORDS[3] = vec4( 0.5, -0.5, 0.0, 1.0);
   NDC_COORDS[4] = vec4( 0.0,  0.5, 0.0, 1.0);
   NDC_COORDS[5] = vec4( 1.0,  0.5, 0.0, 1.0);
   vec4 COLORS[3];
   COLORS[0] = vec4(1.0, 0.0, 0.0, 1.0);
   COLORS[1] = vec4(0.0, 1.0, 0.0, 1.0);
   COLORS[2] = vec4(0.0, 0.0, 1.0, 1.0);
   gl_Position = NDC_COORDS[gl_VertexID];
   color = COLORS[gl_VertexID % 3];
}