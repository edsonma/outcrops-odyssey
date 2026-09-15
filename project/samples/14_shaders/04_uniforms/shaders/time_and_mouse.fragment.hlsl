Texture2D tex_scene : register(t0, space2);

cbuffer Uniforms : register(b0, space3) {
  int tick_count;
  float mouse_x;
  float mouse_y;
};

struct Input {
  float4 tex_color : COLOR0;
  float2 tex_coord : TEXCOORD0;
};

struct Output {
  float4 frag_color : SV_Target;
};

Output main(Input input) {
  Output output;
  output.frag_color = float4(abs(sin(tick_count / 60.0)),
                             mouse_x,
                             mouse_y,
                             1.0);
  return output;
}
