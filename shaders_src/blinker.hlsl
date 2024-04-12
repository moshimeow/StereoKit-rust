#include "stereokit.hlsli"

//--color:color = 1, 0, 0, 1
//--tex_trans   = 0,0,1,1
//--diffuse     = white

float4 color;
float        tex_scale;
Texture2D    diffuse   : register(t0);
SamplerState diffuse_s : register(s0);


struct vsIn {
    float4 pos    : SV_Position;
    float3 normal : NORMAL0;
    float2 uv     : TEXCOORD0;
    float4 col    : COLOR0;
};
struct psIn {
    float4 pos       : SV_Position;
    float2 uv        : TEXCOORD0;
    float3 world_pos : TEXCOORD1;
    float3 model_pos : TEXCOORD2;
    float3 normal    : NORMAL0;
    float4 color     : COLOR0;
    uint view_id : SV_RenderTargetArrayIndex;
};

psIn vs(vsIn input, uint id : SV_InstanceID) {
    psIn o;
    o.view_id = id % sk_view_count;
    id        = id / sk_view_count;

float4x4 world_mat = sk_inst[id].world;
    float3   scale     = float3(
        length(float3(world_mat._11,world_mat._12,world_mat._13)),
        length(float3(world_mat._21,world_mat._22,world_mat._23)),
        length(float3(world_mat._31,world_mat._32,world_mat._33)));
        
    o.model_pos = input.pos.xyz * scale;
    o.world_pos = mul(float4(input.pos.xyz, 1), world_mat).xyz;
    o.pos       = mul(float4(o.world_pos,   1), sk_viewproj[o.view_id]);
    o.uv        = input.uv * tex_scale;
    o.color     = input.col * color * sk_inst[id].color * abs(sin(sk_time % 100));
    return o;
}


float4 ps(psIn input) : SV_TARGET {
    float4 col     = diffuse.Sample(diffuse_s, input.uv);
    return col * input.color;
}
