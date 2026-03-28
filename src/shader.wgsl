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
	return base_color;
}
