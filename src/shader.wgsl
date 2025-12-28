struct Immediate {
	m_position: mat4x3<f32>,
	m_normal: mat3x3<f32>,
	projection_scale: vec4<f32>,
}

struct Material {
	base_color_factor: vec4<f32>,
	base_color_texcoord: u32,
}

struct VertexToFragment {
	@builtin(position) builtin_position: vec4<f32>,
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) @interpolate(perspective, sample) texcoord_0: vec2<f32>,
	@location(3) @interpolate(perspective, sample) texcoord_1: vec2<f32>,
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

fn hash4(pos: vec3<f32>, sample_i: u32) -> f32 {
	var h = 0u;

	for (var i = 0u; i < 4u; i++) {
		var k = select(bitcast<u32>(pos[i]), sample_i, i == 3u);
		k *= 0xcc9e2d51u;
		k = rotl32(k, 15u);
		k *= 0x1b873593u;
		h ^= k;
		h = rotl32(h, 13u);
		h = h * 5u + 0xe6546b64u;
	}

    return 0x1.0p-32 * f32(fmix32(h));
}

fn hash2(pos: vec2<f32>) -> f32 {
	return 0x1.0p-32 * f32(fmix32(u32(pos.x) ^ (u32(pos.y) << 16u)));
}

var<immediate> imm: Immediate;
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
	vtf.position = imm.m_position * vec4(position, 1.0);
	vtf.normal = imm.m_normal * normal;
	vtf.texcoord_0 = texcoord_0;
	vtf.texcoord_1 = texcoord_1;
	vtf.builtin_position = (imm.projection_scale * vec4(vtf.position, 1.0)).xywz;
	return vtf;
}

@fragment fn fs_main(vtf: VertexToFragment, @builtin(sample_index) sample_index: u32) -> @location(0) vec4<f32> {
	let base_color = material.base_color_factor * textureSample(
		base_color_texture, base_color_sampler,
		select(vtf.texcoord_0, vtf.texcoord_1, material.base_color_texcoord > 0)
	);

	/*
	if hash4(vtf.position, sample_index) >= base_color.w {
		discard;
	}
	return vec4(base_color.xyz, 1.0);
	*/

	return vec4(base_color.xyz, -0.125 + 0.25 * hash2(vtf.builtin_position.xy) + base_color.w);
}
