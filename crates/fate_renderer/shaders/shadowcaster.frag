#version 450

const uint NO_TEXTURE_ID = 255;
const uint ALPHA_MODE_MASK = 1;
const float ALPHA_CUTOFF_BIAS = 0.0000001;

//layout(location = 0) in vec3 oViewSpaceNormal;
layout(location = 0) in vec2 oTexcoords0;
layout(location = 1) in vec2 oTexcoords1;
layout(location = 2) in float oAlpha;
layout(location = 3) in vec4 clipPos;

layout(push_constant) uniform MaterialUniform {
    float alpha;
    uint colorTextureChannel;
    uint alphaMode;
    float alphaCutoff;
} material;

layout(binding = 3, set = 1) uniform sampler2D colorSampler;

layout(location = 0) out vec4 outColor;

vec2 getUV(uint texChannel) {
    if (texChannel == 0) {
        return oTexcoords0;
    }
    return oTexcoords1;
}

float getAlpha(uint textureChannel) {
    float alpha = material.alpha;
    if(textureChannel != NO_TEXTURE_ID) {
        vec2 uv = getUV(textureChannel);
        float sampledAlpha = texture(colorSampler, uv).a;
        alpha *= sampledAlpha;
    }
    return alpha * oAlpha;
}

vec3 getColor(uint textureChannel) {
    vec3 color = vec3(1.0,1.0,1.0);
    if(textureChannel != NO_TEXTURE_ID) {
        vec2 uv = getUV(textureChannel);
        color = texture(colorSampler, uv).rgb;
    }
    return color;
}

bool isMasked(float alpha) {
    return material.alphaMode == ALPHA_MODE_MASK && alpha + ALPHA_CUTOFF_BIAS < material.alphaCutoff;
}

void main() {
    float alpha = getAlpha(material.colorTextureChannel);
    if (isMasked(alpha)) {
        discard;
    }

    //测试用
    vec3 color = getColor(material.colorTextureChannel);
    outColor = vec4(clipPos.z,0.0,0.0,1.0);
}
