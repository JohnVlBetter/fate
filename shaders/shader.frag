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

    // [0-7] Color texture channel
    // [8-15] metallic/roughness texture channel
    // [16-23] emissive texture channel
    // [24-31] normals texture channel
    uint colorMetallicRoughnessEmissiveNormalTextureChannels;

    // [0-7] Occlusion texture channel
    // [8-15] Alpha mode
    // [16-23] Unlit flag
    // [24-31] Workflow (metallic/roughness or specular/glossiness)
    uint occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow;
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

const uint METALLIC_ROUGHNESS_WORKFLOW = 0;
const vec3 DIELECTRIC_SPECULAR = vec3(0.04);
const float PI = 3.1415926;
const uint NO_TEXTURE = 255;

struct TextureChannels {
    uint color;
    uint material;
    uint emissive;
    uint normal;
    uint occlusion;
};

TextureChannels getTextureChannels() {
    return TextureChannels(
        (pcs.colorMetallicRoughnessEmissiveNormalTextureChannels >> 24) & 255,
        (pcs.colorMetallicRoughnessEmissiveNormalTextureChannels >> 16) & 255,
        (pcs.colorMetallicRoughnessEmissiveNormalTextureChannels >> 8) & 255,
        pcs.colorMetallicRoughnessEmissiveNormalTextureChannels & 255,
        (pcs.occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow >> 24) & 255
    );
}

vec3 get_normal(TextureChannels textureChannels) {
    vec3 normalWS = normalize(inNormalWS);

    if (textureChannels.normal != NO_TEXTURE) {
        vec3 normalVal = texture(normalSampler, uv).rgb * 2.0 - 1.0;
        normalWS = normalize(TBN * normalVal);
    }
    
    if (!gl_FrontFacing) {
        normalWS *= -1.0;
    }

    return normalWS;
}

float getRoughness(TextureChannels textureChannels, bool metallicRoughnessWorkflow) {
    float roughness = pcs.emissiveAndRoughnessGlossiness.a;
    if(textureChannels.material != NO_TEXTURE) {
        if (metallicRoughnessWorkflow) {
            roughness *= texture(materialSampler, uv).g;
        } else {
            roughness *= texture(materialSampler, uv).a;
        }
    }

    if (metallicRoughnessWorkflow) {
        return roughness;
    }
    return (1 - roughness);
}

vec3 getEmissiveColor(TextureChannels textureChannels) {
    vec3 emissive = pcs.emissiveAndRoughnessGlossiness.rgb;
    if(textureChannels.emissive != NO_TEXTURE) {
        emissive *= texture(emissiveSampler, uv).rgb;
    }
    return emissive/* * pcs.emissiveIntensity*/;
}

vec4 getBaseColor(TextureChannels textureChannels) {
    vec4 color = pcs.base_color;
    if(textureChannels.color != NO_TEXTURE) {
        color *= vec4(texture(albedoSampler, uv).rgb, 1.0);
    }
    return color;
}

float getMetallic(TextureChannels textureChannels) {
    float metallic = pcs.metallicSpecularAndOcclusion.r;
    if(textureChannels.material != NO_TEXTURE) {
        metallic *= texture(materialSampler, uv).b;
    }
    return metallic;
}

vec3 getSpecular(TextureChannels textureChannels) {
    vec3 specular = pcs.metallicSpecularAndOcclusion.rgb;
    if(textureChannels.material != NO_TEXTURE) {
        specular *= texture(materialSampler, uv).rgb;
    }
    return specular;
}

bool isMetallicRoughnessWorkflow() {
    uint workflow = pcs.occlusionTextureChannelAlphaModeUnlitFlagAndWorkflow & 255;
    if (workflow == METALLIC_ROUGHNESS_WORKFLOW) {
        return true;
    }
    return false;
}

float convertMetallic(vec3 diffuse, vec3 specular, float maxSpecular) {
    const float c_MinRoughness = 0.04;
    float perceivedDiffuse = sqrt(0.299 * diffuse.r * diffuse.r + 0.587 * diffuse.g * diffuse.g + 0.114 * diffuse.b * diffuse.b);
    float perceivedSpecular = sqrt(0.299 * specular.r * specular.r + 0.587 * specular.g * specular.g + 0.114 * specular.b * specular.b);
    if (perceivedSpecular < c_MinRoughness) {
        return 0.0;
    }
    float a = c_MinRoughness;
    float b = perceivedDiffuse * (1.0 - maxSpecular) / (1.0 - c_MinRoughness) + perceivedSpecular - 2.0 * c_MinRoughness;
    float c = c_MinRoughness - perceivedSpecular;
    float D = max(b * b - 4.0 * a * c, 0.0);
    return clamp((-b + sqrt(D)) / (2.0 * a), 0.0, 1.0);
}

vec3 f(vec3 f0, vec3 v, vec3 h) {
    return f0 + (1.0 - f0) * pow(1.0 - max(dot(v, h), 0.0), 5.0);
}

vec3 f(vec3 f0, vec3 v, vec3 n, float roughness) {
    return f0 + (max(vec3(1.0 - roughness), f0) - f0) * pow(1.0 - max(dot(v, n), 0.0), 5.0);
}

float vis(vec3 n, vec3 l, vec3 v, float a) {
    float aa = a * a;
    float nl = max(dot(n, l), 0.0);
    float nv = max(dot(n, v), 0.0);
    float denom = ((nl * sqrt(nv * nv * (1 - aa) + aa)) + (nv * sqrt(nl * nl * (1 - aa) + aa))); 

    if (denom < 0.0) {
        return 0.0;
    }
    return 0.5 / denom;
}

float d(float a, vec3 n, vec3 h) {
    float aa = a * a;
    float nh = max(dot(n, h), 0.0);
    float denom = nh * nh * (aa - 1) + 1;

    return aa / (PI * denom * denom);
}

vec3 PBRColor(
    bool isMetallicRoughnessWorkflow,
    vec3 baseColor,
    float metallic,
    float roughness,
    vec3 specular,
    vec3 n,
    vec3 l,
    vec3 v,
    vec3 h,
    vec3 lightColor,
    float lightIntensity
) {
    vec3 color = vec3(0.0);
    if (dot(n, l) > 0.0 || dot(n, v) > 0.0) {
        vec3 cDiffuse;
        vec3 f0;
        if (isMetallicRoughnessWorkflow) {
            cDiffuse = mix(baseColor * (1.0 - DIELECTRIC_SPECULAR.r), vec3(0.0), metallic);
            f0 = mix(DIELECTRIC_SPECULAR, baseColor, metallic);
        } else {
            cDiffuse = baseColor * (1.0 - max(specular.r, max(specular.g, specular.b)));
            f0 = specular;
        }

        float a = roughness * roughness;

        vec3 f = f(f0, v, h);
        float vis = vis(n, l, v, a);
        float d = d(a, n, h);

        vec3 diffuse = cDiffuse / PI;
        vec3 fDiffuse = (1 - f) * diffuse;
        vec3 fSpecular = max(f * vis * d, 0.0);
        color = max(dot(n, l), 0.0) * (fDiffuse + fSpecular) * lightColor * lightIntensity;
    }
    return color;
}

void main() {
    TextureChannels textureChannels = getTextureChannels();

    vec4 albedo = getBaseColor(textureChannels);
    vec3 normalWS = get_normal(textureChannels);
    vec3 emissive = getEmissiveColor(textureChannels);
    vec3 viewDir = normalize(camera_pos.xyz - positionWS.xyz);
    vec3 lightDir = -normalize(main_light_direction.xyz);

    bool isMetallicRoughnessWorkflow = isMetallicRoughnessWorkflow();
    vec3 specular = getSpecular(textureChannels);
    float roughness = getRoughness(textureChannels, isMetallicRoughnessWorkflow);

    float metallic;
    if (isMetallicRoughnessWorkflow) {
        metallic = getMetallic(textureChannels);
    } else {
        float maxSpecular = max(specular.r, max(specular.g, specular.b));
        metallic = convertMetallic(albedo.rgb, specular, maxSpecular);
    }

    vec3 h = normalize(lightDir + viewDir);

    vec3 color = PBRColor(isMetallicRoughnessWorkflow, albedo.rgb, metallic, roughness, specular, normalWS,
        lightDir, viewDir, h, main_light_color.rgb, main_light_direction.w);
    
    outColor = vec4(emissive + color, 1.0);
}