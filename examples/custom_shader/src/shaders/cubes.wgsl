struct Uniforms {
    projection: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_color: vec4<f32>,
}

const LIGHT_POS: vec3<f32> = vec3<f32>(0.0, 3.0, 3.0);

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var sky_texture: texture_cube<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var normal_texture: texture_2d<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) uv: vec2<f32>,
}

struct Cube {
    @location(4) matrix_0: vec4<f32>,
    @location(5) matrix_1: vec4<f32>,
    @location(6) matrix_2: vec4<f32>,
    @location(7) matrix_3: vec4<f32>,
    @location(8) normal_matrix_0: vec3<f32>,
    @location(9) normal_matrix_1: vec3<f32>,
    @location(10) normal_matrix_2: vec3<f32>,
}

struct Output {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tangent_pos: vec3<f32>,
    @location(2) tangent_camera_pos: vec3<f32>,
    @location(3) tangent_light_pos: vec3<f32>,
}

@vertex
fn vs_main(vertex: Vertex, cube: Cube) -> Output {
     let cube_matrix = mat4x4<f32>(
         cube.matrix_0,
         cube.matrix_1,
         cube.matrix_2,
         cube.matrix_3,
     );

    let normal_matrix = mat3x3<f32>(
        cube.normal_matrix_0,
        cube.normal_matrix_1,
        cube.normal_matrix_2,
    );

    //convert to tangent space to calculate lighting in same coordinate space as normal map sample
    let tangent = normalize(normal_matrix * vertex.tangent);
    let normal = normalize(normal_matrix * vertex.normal);
    let bitangent = cross(tangent, normal);

    //shift everything into tangent space
    let tbn = transpose(mat3x3<f32>(tangent, bitangent, normal));

    let world_pos = cube_matrix * vec4<f32>(vertex.position, 1.0);

    var out: Output;
    out.clip_pos = uniforms.projection * world_pos;
    out.uv = vertex.uv;
    out.tangent_pos = tbn * world_pos.xyz;
    out.tangent_camera_pos = tbn * uniforms.camera_pos.xyz;
    out.tangent_light_pos = tbn * LIGHT_POS;

    return out;
}

//cube properties
const CUBE_BASE_COLOR: vec4<f32> = vec4<f32>(0.294118, 0.462745, 0.611765, 0.6);
const SHINE_DAMPER: f32 = 1.0;
const REFLECTIVITY: f32 = 0.8;
const REFRACTION_INDEX: f32 = 1.31;

//fog, for the ~* cinematic effect *~
const FOG_DENSITY: f32 = 0.15;
const FOG_GRADIENT: f32 = 8.0;
const FOG_COLOR: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);

@fragment
fn fs_main(in: Output) -> @location(0) vec4<f32> {
    let to_camera = in.tangent_camera_pos - in.tangent_pos;

    //normal sample from texture
    var normal = textureSample(normal_texture, tex_sampler, in.uv).xyz;
    normal = normal * 2.0 - 1.0;

    //diffuse
    let dir_to_light: vec3<f32> = normalize(in.tangent_light_pos - in.tangent_pos);
    let brightness = max(dot(normal, dir_to_light), 0.0);
    let diffuse: vec3<f32> = brightness * uniforms.light_color.xyz;

    //specular
    let dir_to_camera = normalize(to_camera);
    let light_dir = -dir_to_light;
    let reflected_light_dir = reflect(light_dir, normal);
    let specular_factor = max(dot(reflected_light_dir, dir_to_camera), 0.0);
    let damped_factor = pow(specular_factor, SHINE_DAMPER);
    let specular: vec3<f32> = damped_factor * uniforms.light_color.xyz * REFLECTIVITY;

    //fog
    let distance = length(to_camera);
    let visibility = clamp(exp(-pow((distance * FOG_DENSITY), FOG_GRADIENT)), 0.0, 1.0);

    //reflection
    let reflection_dir = reflect(dir_to_camera, normal);
    let reflection_color = textureSample(sky_texture, tex_sampler, reflection_dir);
    let refraction_dir = refract(dir_to_camera, normal, REFRACTION_INDEX);
    let refraction_color = textureSample(sky_texture, tex_sampler, refraction_dir);
    let final_reflect_color = mix(reflection_color, refraction_color, 0.5);

    //mix it all together!
    var color = vec4<f32>(CUBE_BASE_COLOR.xyz * diffuse + specular, CUBE_BASE_COLOR.w);
    color = mix(color, final_reflect_color, 0.8);
    color = mix(FOG_COLOR, color, visibility);

    return color;
}
