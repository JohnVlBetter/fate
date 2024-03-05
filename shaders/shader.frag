#version 450

layout(binding = 1) uniform sampler2D albedoSampler;
layout(binding = 2) uniform sampler2D normalSampler;
layout(binding = 3) uniform sampler2D materialSampler;
layout(binding = 4) uniform sampler2D aoSampler;
layout(binding = 5) uniform sampler2D emissiveSampler;

layout(push_constant) uniform PushConstants {
    layout(offset = 64) float opacity;
} pcs;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec4 posWS;
layout(location = 3) in vec3 normalWS;
layout(location = 4) in vec4 main_light_direction;
layout(location = 5) in vec4 main_light_color;
layout(location = 6) in vec4 camera_pos;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 nor_normalWS = normalize(normalWS);
    vec3 lightDir = normalize(-main_light_direction.rgb);
    vec4 albedo = vec4(texture(materialSampler, fragTexCoord).rgb, 1.0);
    vec3 viewDir = normalize(camera_pos.xyz - posWS.xyz);
    vec3 h = normalize(viewDir + lightDir);

    vec3 diffuse = clamp(dot(lightDir,nor_normalWS),0.0,1.0) * main_light_color.rgb * albedo.rgb;
    vec3 specular = pow(clamp(dot(h, nor_normalWS),0.0,1.0), 32.0) * main_light_color.rgb;

    outColor = vec4(diffuse + specular, 1.0);
}