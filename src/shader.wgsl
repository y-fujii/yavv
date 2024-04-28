struct VsConstants {
	transform: mat4x4<f32>,
	projection: mat4x4<f32>,
}

struct Material {
	base_color_factor: vec4<f32>,
	base_color_texcoord: u32,
}

struct VertexToFragment {
	@builtin(position) position: vec4<f32>,
	@location(0) normal: vec3<f32>,
	@location(1) @interpolate(perspective, sample) texcoord_0: vec2<f32>,
	@location(2) @interpolate(perspective, sample) texcoord_1: vec2<f32>,
}

fn rotl32(x: u32, r: u32) -> u32 {
	return (x << r) | (x >> (32u - r));
}

fn fmix32(x: u32) -> u32 {
	var h = x;
	h ^= h >> 16u;
	h *= 0x85ebca6bu;
	h ^= h >> 13u;
	h *= 0xc2b2ae35u;
	h ^= h >> 16u;
	return h;
}

fn rand(pos: vec4<f32>, sample_i: u32) -> f32 {
	var h = 0u;

	for (var i = 0u; i < 4u; i++) {
		var k = select(bitcast<u32>(pos[i]), sample_i, i == 3u);
		k *= 0xcc9e2d51u;
		k = rotl32(k, 15u);
		k *= 0x1b873593u;
		h ^= k;
		h = rotl32(h, 13u);
		h = h * 5u + 0xe6546b64;
	}

    return 0x1.0p-32 * f32(fmix32(h));
}

fn rand2(pos: vec2<f32>) -> f32 {
    return 0x1.0p-32 * f32(fmix32(u32(pos.x) ^ (u32(pos.y) << 16u)));
}

fn bayer(i: u32, j: u32) -> u32 {
	return (2 * (i & 1) + 3 * (j & 1)) & 3;
}

var<push_constant> vs_constants: VsConstants;
@group(0) @binding(0) var<uniform> material: Material;
@group(0) @binding(1) var base_color_texture: texture_2d<f32>;
@group(0) @binding(2) var base_color_sampler: sampler;

@vertex fn vs_main(
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) texcoord_0: vec2<f32>,
	@location(3) texcoord_1: vec2<f32>
) -> VertexToFragment {
	var vtf: VertexToFragment;
	vtf.position = vs_constants.projection * vs_constants.transform * vec4(position, 1.0);
	vtf.normal = (vs_constants.transform * vec4(normal, 0.0)).xyz; // XXX
	vtf.texcoord_0 = texcoord_0;
	vtf.texcoord_1 = texcoord_1;
	return vtf;
}

@fragment fn fs_main(vtf: VertexToFragment, @builtin(sample_index) sample_index: u32) -> @location(0) vec4<f32> {
	let base_color = material.base_color_factor * textureSample(
		base_color_texture, base_color_sampler,
		select(vtf.texcoord_0, vtf.texcoord_1, material.base_color_texcoord > 0)
	);

	/*
	let bayer = vec4(2, 0, 3, 1);
	let r = 4 * bayer[sample_index] + bayer[(u32(vtf.position.x) & 1) + 2 * (u32(vtf.position.y) & 1)];
	//let r = 4 * bayer(sample_index, sample_index / 2) + bayer(u32(vtf.position.x), u32(vtf.position.y));
	if (1.0 / 32.0) * f32(1 + 2 * r) >= base_color.w {
		discard;
	}
	*/
	/*
	if rand(vtf.position, sample_index) >= base_color.w {
		discard;
	}
	return vec4(base_color.xyz, 1.0);
	*/

	return vec4(base_color.xyz, base_color.w + (0.25 * rand2(vtf.position.xy) - 0.125));
}
