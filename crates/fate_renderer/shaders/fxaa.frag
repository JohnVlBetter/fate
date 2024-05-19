#version 450

layout(location = 0) in vec2 oCoords;

layout(binding = 0) uniform sampler2D srcImage;

layout(location = 0) out vec4 outColor;

layout(constant_id = 0) const uint FXAA_MODE = 0;

const uint FXAA_QUALITY = 0;
const uint FXAA_CONSOLE = 1;

#define EXTRA_EDGE_STEPS 10
#define EDGE_STEP_SIZES 1.0, 1.0, 1.0, 1.0, 1.5, 2.0, 2.0, 2.0, 2.0, 4.0
#define LAST_EDGE_STEP_GUESS 8.0

layout(push_constant) uniform Constants {
    float AbsoluteLuminanceThreshold;
    float RelativeLuminanceThreshold;
    float SubpixelBlending;
} c;

float linearRgbToLuminance(vec3 linearRgb)
{
    return dot(linearRgb, vec3(0.2126729f,  0.7151522f, 0.0721750f));
}

vec4 getSource(vec2 screenUV) {
	return texture(srcImage, screenUV);
}

float getLuminance(vec2 uv, float uOffset, float vOffset) {
    vec2 texelSize = textureSize(srcImage, 0).xy;
	uv += vec2(uOffset, vOffset) / texelSize;
	return linearRgbToLuminance(getSource(uv).rgb);
}

struct LuminanceNeighborhood {
	float m, n, e, s, w, ne, se, sw, nw;
	float highest, lowest, range;
};

LuminanceNeighborhood getLuminanceNeighborhood (vec2 uv) {
	LuminanceNeighborhood luminance;
	luminance.m = getLuminance(uv, 0.0, 0.0);
	luminance.n = getLuminance(uv, 0.0, 1.0);
	luminance.e = getLuminance(uv, 1.0, 0.0);
	luminance.s = getLuminance(uv, 0.0, -1.0);
	luminance.w = getLuminance(uv, -1.0, 0.0);
	luminance.ne = getLuminance(uv, 1.0, 1.0);
	luminance.se = getLuminance(uv, 1.0, -1.0);
	luminance.sw = getLuminance(uv, -1.0, -1.0);
	luminance.nw = getLuminance(uv, -1.0, 1.0);
	luminance.highest = max(max(max(max(luminance.m, luminance.n), luminance.e), luminance.s), luminance.w);
	luminance.lowest = min(min(min(min(luminance.m, luminance.n), luminance.e), luminance.s), luminance.w);
	luminance.range = luminance.highest - luminance.lowest;
	return luminance;
}

bool shouldSkipPixel (LuminanceNeighborhood luminance) {
	return luminance.range < max(c.AbsoluteLuminanceThreshold, c.RelativeLuminanceThreshold * luminance.highest);
}

bool isHorizontalEdge (LuminanceNeighborhood luminance) {
	float horizontal =
		2.0 * abs(luminance.n + luminance.s - 2.0 * luminance.m) +
		abs(luminance.ne + luminance.se - 2.0 * luminance.e) +
		abs(luminance.nw + luminance.sw - 2.0 * luminance.w);
	float vertical =
		2.0 * abs(luminance.e + luminance.w - 2.0 * luminance.m) +
		abs(luminance.ne + luminance.nw - 2.0 * luminance.n) +
		abs(luminance.se + luminance.sw - 2.0 * luminance.s);
	return horizontal >= vertical;
}

struct FXAAEdge {
	bool isHorizontal;
	float pixelStep;
	float luminanceGradient, otherluminance;
};

FXAAEdge getFXAAEdge (LuminanceNeighborhood luminance) {
	FXAAEdge edge;
	edge.isHorizontal = isHorizontalEdge(luminance);
	float luminanceP, luminanceN;
    vec2 texelSize = textureSize(srcImage, 0).xy;
	if (edge.isHorizontal) {
		edge.pixelStep = 1.0 / texelSize.y;
		luminanceP = luminance.n;
		luminanceN = luminance.s;
	}
	else {
		edge.pixelStep = 1.0 / texelSize.x;
		luminanceP = luminance.e;
		luminanceN = luminance.w;
	}
	float gradientP = abs(luminanceP - luminance.m);
	float gradientN = abs(luminanceN - luminance.m);
	if (gradientP < gradientN) {
		edge.pixelStep = -edge.pixelStep;
		edge.luminanceGradient = gradientN;
		edge.otherluminance = luminanceN;
	}
	else {
		edge.luminanceGradient = gradientP;
		edge.otherluminance = luminanceP;
	}
	
	return edge;
}

float getSubpixelBlendFactor (LuminanceNeighborhood luminance) {
	float mfilter = 2.0 * (luminance.n + luminance.e + luminance.s + luminance.w);
	mfilter += luminance.ne + luminance.nw + luminance.se + luminance.sw;
	mfilter *= 1.0 / 12.0;
	mfilter = abs(mfilter - luminance.m);
	mfilter = clamp(mfilter / luminance.range, 0.0, 1.0);
	mfilter = smoothstep(0, 1, mfilter);
	return mfilter * mfilter * c.SubpixelBlending;
}

const float edgeStepSizes[EXTRA_EDGE_STEPS] = { EDGE_STEP_SIZES };

float getEdgeBlendFactor (LuminanceNeighborhood luminance, FXAAEdge edge, vec2 uv) {
	vec2 edgeUV = uv;
	vec2 uvStep = vec2(0.0, 0.0);
    vec2 texelSize = textureSize(srcImage, 0).xy;
	if (edge.isHorizontal) {
		edgeUV.y += 0.5 * edge.pixelStep;
		uvStep.x = 1.0 / texelSize.x;
	}
	else {
		edgeUV.x += 0.5 * edge.pixelStep;
		uvStep.y = 1.0 / texelSize.y;
	}
	float edgeluminance = 0.5 * (luminance.m + edge.otherluminance);
	float gradientThreshold = 0.25 * edge.luminanceGradient;
			
	vec2 uvP = edgeUV + uvStep;
	float luminanceDeltaP = getLuminance(uvP, 0.0, 0.0) - edgeluminance;
	bool atEndP = abs(luminanceDeltaP) >= gradientThreshold;
	for (int i = 0; i < EXTRA_EDGE_STEPS && !atEndP; i++) {
		uvP += uvStep * edgeStepSizes[i];
		luminanceDeltaP = getLuminance(uvP, 0.0, 0.0) - edgeluminance;
		atEndP = abs(luminanceDeltaP) >= gradientThreshold;
	}
	if (!atEndP) {
		uvP += uvStep * LAST_EDGE_STEP_GUESS;
	}
	vec2 uvN = edgeUV - uvStep;
	float luminanceDeltaN = getLuminance(uvN, 0.0, 0.0) - edgeluminance;
	bool atEndN = abs(luminanceDeltaN) >= gradientThreshold;
	for (int i = 0; i < EXTRA_EDGE_STEPS && !atEndN; i++) {
		uvN -= uvStep * edgeStepSizes[i];
		luminanceDeltaN = getLuminance(uvN, 0.0, 0.0) - edgeluminance;
		atEndN = abs(luminanceDeltaN) >= gradientThreshold;
	}
	if (!atEndN) {
		uvN -= uvStep * LAST_EDGE_STEP_GUESS;
	}
	float distanceToEndP, distanceToEndN;
	if (edge.isHorizontal) {
		distanceToEndP = uvP.x - uv.x;
		distanceToEndN = uv.x - uvN.x;
	}
	else {
		distanceToEndP = uvP.y - uv.y;
		distanceToEndN = uv.y - uvN.y;
	}
	float distanceToNearestEnd;
	bool deltaSign;
	if (distanceToEndP <= distanceToEndN) {
		distanceToNearestEnd = distanceToEndP;
		deltaSign = luminanceDeltaP >= 0;
	}
	else {
		distanceToNearestEnd = distanceToEndN;
		deltaSign = luminanceDeltaN >= 0;
	}
	if (deltaSign == (luminance.m - edgeluminance >= 0)) {
		return 0.0;
	}
	else {
		return 0.5 - distanceToNearestEnd / (distanceToEndP + distanceToEndN);
	}
}

void main() {
    if (FXAA_MODE == FXAA_QUALITY)
    {
        LuminanceNeighborhood luminance = getLuminanceNeighborhood(oCoords);
        if (!shouldSkipPixel(luminance)) 
        {
            FXAAEdge edge = getFXAAEdge(luminance);
	        float blendFactor = max(
                getSubpixelBlendFactor(luminance), getEdgeBlendFactor (luminance, edge, oCoords)
	        );
	        vec2 blendUV = oCoords;
	        if (edge.isHorizontal) {
	        	blendUV.y += blendFactor * edge.pixelStep;
	        }
	        else {
	        	blendUV.x += blendFactor * edge.pixelStep;
	        }
            outColor = getSource(blendUV);
	    }
        else
        {
	    	outColor = getSource(oCoords);
        }
    }
    else 
    {
        //FXAA_CONSOLE先当关闭用，没实现速度版本，只有高质量版本
	    outColor = getSource(oCoords);
    }
}