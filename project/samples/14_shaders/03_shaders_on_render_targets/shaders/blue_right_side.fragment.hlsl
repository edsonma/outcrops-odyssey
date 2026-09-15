Texture2D tex_scene : register(t0, space2);
SamplerState sam_scene : register(s0, space2);

struct Input {
  float4 tex_color : COLOR0;
  float2 tex_coord : TEXCOORD0;
};

struct Output {
  float4 frag_color : SV_Target;
};

Output main(Input input) {
  Output output;
  float4 color = tex_scene.Sample(sam_scene, input.tex_coord) * input.tex_color;
  if (input.tex_coord.x > 0.5) {
    output.frag_color = float4(color.r, color.g, color.b, color.a);
  } else {
    output.frag_color = float4(0, 0, color.b, color.a);
  }
  return output;
}
