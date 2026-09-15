Texture2D scene : register(t0, space2);
Texture2D displacement_map : register(t1, space2);
SamplerState sampler0 : register(s0, space2);
SamplerState sampler1 : register(s1, space2);

struct Input {
    float4 tex_color : COLOR0;
    float2 tex_coord : TEXCOORD0;
};

struct Output {
    float4 frag_color : SV_Target;
};

Output main(Input input) {
    Output output;
    float4 tex1_color = displacement_map.Sample(sampler1, input.tex_coord);

    float displacement_perc = 0.02;
    float2 displacement_uv = float2(
        input.tex_coord.x + (tex1_color.r * displacement_perc - displacement_perc / 2.0),
        input.tex_coord.y + (tex1_color.r * displacement_perc - displacement_perc / 2.0)
    );
    float2 resolved_uv = float2(
        clamp(displacement_uv.x, 0.0, 1.0),
        clamp(displacement_uv.y, 0.0, 1.0)
    );
    output.frag_color = scene.Sample(sampler0, resolved_uv) * input.tex_color;
    return output;
}
