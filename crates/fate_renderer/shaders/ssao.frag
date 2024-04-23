#version 450

layout (constant_id = 0) const uint SSAO_KERNEL_SIZE = 32;

layout(push_constant) uniform Config {
    float ssaoRadius;
    float ssaoStrength;
} config;

layout(location = 0) in vec2 oCoords;
layout(location = 1) in vec3 oViewRay;

layout(binding = 0, set = 0) uniform sampler2D normalsSampler;
layout(binding = 1, set = 0) uniform sampler2D depthSampler;
layout(binding = 2, set = 0) uniform sampler2D noiseSampler;

layout(binding = 3, set = 1) uniform SSAOKernel {
	vec4 samples[SSAO_KERNEL_SIZE];
} ssaoKernel;

layout(binding = 4, set = 2) uniform CameraUBO {
    mat4 view;
    mat4 proj;
    mat4 invertedProj;
    vec4 eye;
    float zNear;
    float zFar;
} cameraUBO;

layout(location = 0) out float finalColor;

float linearDepth(vec2 uv) {
    float near = cameraUBO.zNear;
    float far = cameraUBO.zFar;
    float depth = texture(depthSampler, uv).r;
    return (near * far) / (far + depth * (near - far));
}

void main() {
    vec3 position = oViewRay * linearDepth(oCoords);

    vec3 normal = normalize(texture(normalsSampler, oCoords).xyz);

    ivec2 ssaoSize = textureSize(depthSampler, 0);
    ivec2 noiseSize = textureSize(noiseSampler, 0);
    vec2 noiseScale = vec2(float(ssaoSize.x) / float(noiseSize.x), float(ssaoSize.y) / float(noiseSize.y));
    vec3 randomVec = texture(noiseSampler, oCoords * noiseScale).xyz;

    vec3 tangent = normalize(randomVec - normal * dot(randomVec, normal));
    vec3 bitangent = cross(normal, tangent);
    mat3 tbn = mat3(tangent, bitangent, normal);

    float occlusion = 0.0;
    const float bias = 0.01f;
    for (int i = 0; i < SSAO_KERNEL_SIZE; i++) {
        vec3 kSample = tbn * ssaoKernel.samples[i].xyz;
        kSample = kSample * config.ssaoRadius + position;
        
        vec4 offset = vec4(kSample, 1.0);
        offset = cameraUBO.proj * offset;
        offset.xy /= offset.w;
        offset.xy = offset.xy * 0.5 + 0.5;
        
        float depth = -linearDepth(offset.xy);

        float rangeCheck = smoothstep(0.0f, 1.0f, config.ssaoRadius / abs(depth - position.z));
		occlusion += (depth >= kSample.z + bias ? 1.0f : 0.0f) * rangeCheck;
    }
    occlusion = 1.0 - (occlusion / float(SSAO_KERNEL_SIZE));
    
    finalColor = pow(occlusion, config.ssaoStrength);
}