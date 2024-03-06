#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    vec4 color;
    vec4 main_light_direction;
    vec4 main_light_color;
    vec4 camera_pos;
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pcs;

layout(location = 0) in vec3 positionOS;
layout(location = 1) in vec3 vertex_color;
layout(location = 2) in vec3 normalOS;
layout(location = 3) in vec2 texCoord;
layout(location = 4) in vec4 tangentOS;

layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec2 uv;
layout(location = 2) out vec4 positionWS;
layout(location = 3) out vec3 normalWS;
layout(location = 4) out vec4 main_light_direction;
layout(location = 5) out vec4 main_light_color;
layout(location = 6) out vec4 camera_pos;
layout(location = 7) out mat3 TBN;

void main() {
    positionWS = pcs.model * vec4(positionOS, 1.0);
    normalWS = normalize((pcs.model * vec4(normalOS, 0.0)).xyz);
    vec3 tangentWS = normalize((pcs.model * vec4(tangentOS.xyz, 0.0)).xyz);
    tangentWS = normalize(tangentWS - dot(tangentWS, normalWS) * normalWS);
    vec3 bitangentWS = cross(normalWS, tangentWS) * tangentOS.w;
    TBN = mat3(tangentWS, bitangentWS, normalWS);

    frag_color = tangentOS.rgb;
    uv = texCoord;
    main_light_direction = ubo.main_light_direction;
    main_light_color = ubo.main_light_color;
    camera_pos = ubo.camera_pos;

    gl_Position = ubo.proj * ubo.view * positionWS;
}