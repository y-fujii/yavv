struct VsConstants {
	projection: mat4x4<f32>,
}

struct Material {
	base_color_factor: vec4<f32>,
}

struct VertexToFragment {
	@builtin(position) position: vec4<f32>,
	@location(0) normal: vec3<f32>,
	@location(1) @interpolate(perspective, sample) texcoord_0: vec2<f32>,
}

fn rand(pos: vec4<f32>, i: u32) -> f32 {
	var h = i ^ (u32(pos.x) << 4) ^ (u32(pos.y) << 18) ^ u32(0x1p32 * pos.z);

	// MurmurHash3 fmix32.
	h ^= h >> 16;
	h *= 0x85ebca6bu;
	h ^= h >> 13;
	h *= 0xc2b2ae35u;
	h ^= h >> 16;

    return 0x1.0p-32 * f32(h);
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
	@location(2) texcoord_0: vec2<f32>
) -> VertexToFragment {
	var vtf: VertexToFragment;
	vtf.position = vs_constants.projection * vec4(position, 1.0);
	vtf.normal = normal;
	vtf.texcoord_0 = texcoord_0;
	return vtf;
}

@fragment fn fs_main(vtf: VertexToFragment, @builtin(sample_index) sample_index: u32) -> @location(0) vec4<f32> {
	let base_color = material.base_color_factor * textureSample(base_color_texture, base_color_sampler, vtf.texcoord_0);
	/*
	let bayer = vec4(2, 0, 3, 1);
	let r = 4 * bayer[sample_index] + bayer[(u32(vtf.position.x) & 1) + 2 * (u32(vtf.position.y) & 1)];
	//let r = 4 * bayer(sample_index, sample_index / 2) + bayer(u32(vtf.position.x), u32(vtf.position.y));
	if (1.0 / 32.0) * f32(1 + 2 * r) >= base_color.w {
		discard;
	}
	*/
	if rand(vtf.position, sample_index) >= base_color.w {
		discard;
	}
	return vec4(base_color.xyz, 1.0);
}
