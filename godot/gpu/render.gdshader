shader_type spatial;

// Values resulting from the compute shader
uniform usampler2D computes;

// interpolate between two colors
uniform vec4 color_off : source_color = vec4(0.0);
uniform vec4 color_on  : source_color = vec4(1.0);

const float MAX_VALUE = 1024.0;

// Specify the value to sample in the `computes` buffer
instance uniform uint index;

// No transformations to apply to the vertices
//void vertex() {}

void fragment() {
	uint  texel  = texelFetch(computes, ivec2(int(index), 0), 0).r;
	float factor = min(float(texel) / MAX_VALUE, 1.0);
	ALBEDO = mix(color_off.rgb, color_on.rgb, factor);
}
