struct VsConstants {
	projection: mat4x4<f32>,
}

struct Material {
	base_color_factor: vec4<f32>,
}

struct VertexToFragment {
	@builtin(position) position: vec4<f32>,
	@location(0) normal: vec3<f32>,
	@location(1) texcoord_0: vec2<f32>,
}

var<push_constant> vs_constants: VsConstants;
@group(0) @binding(0) var<uniform> material: Material;
@group(0) @binding(1) var base_color_texture: texture_2d<f32>;
@group(0) @binding(2) var base_color_sampler: sampler;

@vertex
fn vs_main(
	@builtin(vertex_index) index: u32,
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

@fragment
fn fs_main(vtf: VertexToFragment) -> @location(0) vec4<f32> {
	let color = material.base_color_factor * textureSample(base_color_texture, base_color_sampler, vtf.texcoord_0);
	return color;
}
