#version 450

layout(binding = 1) uniform sampler2D albedoSampler;
layout(binding = 2) uniform sampler2D normalSampler;
layout(binding = 3) uniform sampler2D materialSampler;
layout(binding = 4) uniform sampler2D aoSampler;
layout(binding = 5) uniform sampler2D emissiveSampler;

layout(push_constant) uniform PushConstants {
    layout(offset = 64) vec4 base_color;

    // - emissive: emissiveAndRoughnessGlossiness.rgb
    // - roughness: emissiveAndRoughnessGlossiness.a (metallic/roughness workflows)
    // - glossiness: emissiveAndRoughnessGlossiness.a (specular/glossiness workflows)
    vec4 emissiveAndRoughnessGlossiness;

    // - metallic: metallicSpecularAndOcclusion.r (metallic/roughness workflows)
    // - specular: metallicSpecularAndOcclusion.rgb (specular/glossiness workflows)
    // - occlusion: metallicSpecularAndOcclusion.a
    vec4 metallicSpecularAndOcclusion;

    uint workflow;
} pcs;

layout(location = 0) in vec3 frag_color;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 positionWS;
layout(location = 3) in vec3 inNormalWS;
layout(location = 4) in vec4 main_light_direction;
layout(location = 5) in vec4 main_light_color;
layout(location = 6) in vec4 camera_pos;
layout(location = 7) in mat3 TBN;

layout(location = 0) out vec4 outColor;

vec3 get_normal() {
    vec3 normalWS = normalize(inNormalWS);
    vec3 normalVal = texture(normalSampler, uv).rgb * 2.0 - 1.0;
    normalWS = normalize(TBN * normalVal);
    
    if (!gl_FrontFacing) {
        normalWS *= -1.0;
    }

    return normalWS;
}

void main() {
    vec4 albedo = vec4(texture(albedoSampler, uv).rgb, 1.0);
    vec3 normalWS = get_normal();
    vec3 viewDir = normalize(camera_pos.xyz - positionWS.xyz);
    vec3 lightDir = normalize(-main_light_direction.rgb);

    outColor = vec4(pcs.emissiveAndRoughnessGlossiness.rgb, 1.0);
}