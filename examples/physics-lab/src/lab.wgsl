struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
struct Input {
 @location(0) position:vec3<f32>, @location(1) normal:vec3<f32>,
 @location(2) m0:vec4<f32>, @location(3) m1:vec4<f32>, @location(4) m2:vec4<f32>, @location(5) m3:vec4<f32>,
 @location(6) n0:vec4<f32>, @location(7) n1:vec4<f32>, @location(8) n2:vec4<f32>, @location(9) color:vec4<f32>
}
struct Output { @builtin(position) position:vec4<f32>, @location(0) normal:vec3<f32>, @location(1) color:vec4<f32> }
@vertex fn vs_main(v:Input)->Output {
 var o:Output;
 o.position=camera.view_projection*mat4x4<f32>(v.m0,v.m1,v.m2,v.m3)*vec4<f32>(v.position,1.0);
 o.normal=normalize(mat3x3<f32>(v.n0.xyz,v.n1.xyz,v.n2.xyz)*v.normal);o.color=v.color;return o;
}
@fragment fn fs_main(v:Output)->@location(0) vec4<f32> {
 let light=0.3+0.7*max(dot(normalize(v.normal),normalize(vec3<f32>(-0.5,1.0,0.4))),0.0);
 return vec4<f32>(v.color.rgb*light,1.0);
}
