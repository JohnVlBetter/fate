#version 450

layout(location = 0) in vec2 oCoords;

layout(binding = 0) uniform sampler2D inputImage;
layout(binding = 1) uniform sampler2D bloomImage;

layout(location = 0) out vec4 finalColor;

layout(constant_id = 0) const uint TONE_MAP_MODE = 0;
const uint TONE_MAP_MODE_DEFAULT = 0;
const uint TONE_MAP_MODE_UNCHARTED = 1;
const uint TONE_MAP_MODE_HEJL_RICHARD = 2;
const uint TONE_MAP_MODE_ACES = 3;

layout(push_constant) uniform Constants {
    float bloomStrength;
} c;

const float GAMMA = 2.2;
const float INV_GAMMA = 1.0 / GAMMA;

vec3 LINEARtoSRGB(vec3 color) {
    return pow(color, vec3(INV_GAMMA));
}

vec3 toneMapUncharted2Impl(vec3 color) {
    const float A = 0.15;
    const float B = 0.50;
    const float C = 0.10;
    const float D = 0.20;
    const float E = 0.02;
    const float F = 0.30;
    return ((color*(A*color+C*B)+D*E)/(color*(A*color+B)+D*F))-E/F;
}

vec3 toneMapUncharted(vec3 color) {
    const float W = 11.2;
    color = toneMapUncharted2Impl(color * 2.0);
    vec3 whiteScale = 1.0 / toneMapUncharted2Impl(vec3(W));
    return LINEARtoSRGB(color * whiteScale);
}

vec3 toneMapHejlRichard(vec3 color) {
    color = max(vec3(0.0), color - vec3(0.004));
    return (color*(6.2*color+.5))/(color*(6.2*color+1.7)+0.06);
}

vec3 toneMapACES(vec3 color) {
    const float A = 2.51;
    const float B = 0.03;
    const float C = 2.43;
    const float D = 0.59;
    const float E = 0.14;
    return LINEARtoSRGB(clamp((color * (A * color + B)) / (color * (C * color + D) + E), 0.0, 1.0));
}

vec3 defaultToneMap(vec3 color) {
    color = color/(color + 1.0);
    return LINEARtoSRGB(color);
}

float linearDepth(vec2 uv) {
    float near = 0.01;
    float far = 100.0;
    float depth = texture(inputImage, uv).r;
    return (near * far) / (far + depth * (near - far));
}

float LinearizeDepth(float depth)
{
    float near_plane = 0.01;
    float far_plane = 100.0;
    float z = depth * 2.0 - 1.0; // Back to NDC 
    return (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane));	
}

void main() {
    vec3 color = texture(inputImage, oCoords).rgb;
    vec3 bloom = texture(bloomImage, oCoords).rgb;
    vec3 bloomed = mix(color, bloom, c.bloomStrength);
    float depth = linearDepth(oCoords);
    finalColor = vec4(depth,depth,depth, 1.0);
    float d = LinearizeDepth(color.r);
    finalColor = vec4(color.rgb, 1.0);

    /*if (TONE_MAP_MODE == TONE_MAP_MODE_DEFAULT) {
        color = defaultToneMap(bloomed);
    } else if (TONE_MAP_MODE == TONE_MAP_MODE_UNCHARTED) {
        color = toneMapUncharted(bloomed);
    } else if (TONE_MAP_MODE == TONE_MAP_MODE_HEJL_RICHARD) {
        color = toneMapHejlRichard(bloomed);
    } else if (TONE_MAP_MODE == TONE_MAP_MODE_ACES) {
        color = toneMapACES(bloomed);
    } else {
        color = LINEARtoSRGB(bloomed);
    }

    finalColor = vec4(color, 1.0);*/
}
