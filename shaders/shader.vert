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

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inTexCoord;
layout(location = 4) in vec4 inTangent;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) out vec4 posWS;
layout(location = 3) out vec3 normalWS;
layout(location = 4) out vec4 main_light_direction;
layout(location = 5) out vec4 main_light_color;
layout(location = 6) out vec4 camera_pos;

void main() {
    posWS = pcs.model * vec4(inPosition, 1.0);
    normalWS = (pcs.model * vec4(inNormal, 1.0)).rgb;
    gl_Position = ubo.proj * ubo.view * posWS;
    fragColor = inTangent.rgb;
    fragTexCoord = inTexCoord;
    main_light_direction = ubo.main_light_direction;
    main_light_color = ubo.main_light_color;
    camera_pos = ubo.camera_pos;
}