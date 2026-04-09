struct Uniforms {
    // quad_bounds = (x, y, width, height) in normalized screen coords - expanded for blur sampling
    quad_bounds: vec4<f32>,
    // clip_bounds = (x, y, width, height) in normalized screen coords - original widget bounds for SDF
    clip_bounds: vec4<f32>,
    // params.x = blur_radius, params.y = direction (0=horizontal, 1=vertical)
    // params.z = texture_width, params.w = texture_height
    params: vec4<f32>,
    // border_radius = (top_left, top_right, bottom_right, bottom_left) in pixels
    border_radius: vec4<f32>,
    // fade_params.x = fade_start (0.0–1.0 fraction of bounds height)
    fade_params: vec4<f32>,
}

@group(0) @binding(1)
var<uniform> u_uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    let uv = uvs[vertex_index];

    // Use quad_bounds for vertex positioning (expanded area)
    let x = u_uniforms.quad_bounds.x + uv.x * u_uniforms.quad_bounds.z;
    let y = u_uniforms.quad_bounds.y + uv.y * u_uniforms.quad_bounds.w;

    let clip_x = x * 2.0 - 1.0;
    let clip_y = 1.0 - y * 2.0;

    var out: VertexOutput;
    out.position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var u_sampler: sampler;

@group(1) @binding(0)
var u_texture: texture_2d<f32>;

// Compute signed distance to a rounded rectangle
// pos: position relative to rectangle center
// half_size: half of rectangle width/height
// radius: corner radius for this quadrant
fn rounded_rect_sdf(pos: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(pos) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

// Get the appropriate corner radius based on which quadrant the pixel is in
fn get_corner_radius(pos: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // radii = (top_left, top_right, bottom_right, bottom_left)
    if (pos.x < 0.0) {
        if (pos.y < 0.0) {
            return radii.x; // top-left
        } else {
            return radii.w; // bottom-left
        }
    } else {
        if (pos.y < 0.0) {
            return radii.y; // top-right
        } else {
            return radii.z; // bottom-right
        }
    }
}

// Compute Gaussian weight for a given offset and sigma
fn gaussian(x: f32, sigma: f32) -> f32 {
    let coeff = 1.0 / (sqrt(2.0 * 3.14159265) * sigma);
    let exponent = -(x * x) / (2.0 * sigma * sigma);
    return coeff * exp(exponent);
}

@fragment
fn fs_main(
    @builtin(position) frag_pos: vec4<f32>
) -> @location(0) vec4<f32> {
    // Texture dimensions from uniforms
    let tex_width = u_uniforms.params.z;
    let tex_height = u_uniforms.params.w;
    let radius = u_uniforms.params.x;
    let direction = u_uniforms.params.y; // 0 = horizontal, 1 = vertical

    // Compute SDF alpha for clip_bounds (original widget bounds).
    let has_border_radius = u_uniforms.border_radius.x > 0.0 || u_uniforms.border_radius.y > 0.0 ||
                            u_uniforms.border_radius.z > 0.0 || u_uniforms.border_radius.w > 0.0;

    let bounds_px = vec4<f32>(
        u_uniforms.clip_bounds.x * tex_width,
        u_uniforms.clip_bounds.y * tex_height,
        u_uniforms.clip_bounds.z * tex_width,
        u_uniforms.clip_bounds.w * tex_height
    );

    let rect_center = vec2<f32>(
        bounds_px.x + bounds_px.z * 0.5,
        bounds_px.y + bounds_px.w * 0.5
    );
    let half_size = vec2<f32>(bounds_px.z * 0.5, bounds_px.w * 0.5);
    let pos = vec2<f32>(frag_pos.x, frag_pos.y) - rect_center;

    let corner_radius = select(0.0, get_corner_radius(pos, half_size, u_uniforms.border_radius), has_border_radius);
    let dist = rounded_rect_sdf(pos, half_size, corner_radius);
    var sdf_alpha = 1.0 - smoothstep(-0.5, 0.5, dist);

    // Apply vertical fade: full opacity above fade_start, linear fade to 0 at bottom.
    let fade_start = u_uniforms.fade_params.x;
    if (fade_start < 1.0) {
        let bounds_top_px = u_uniforms.clip_bounds.y * tex_height;
        let bounds_height_px = u_uniforms.clip_bounds.w * tex_height;
        let local_y = (frag_pos.y - bounds_top_px) / bounds_height_px;
        var fade_alpha = 1.0 - smoothstep(fade_start, 1.0, local_y);
        // Invert fade for restore pass: output (1-fade)*sdf to additively
        // blend the original scene back in the fade region.
        if (u_uniforms.fade_params.y > 0.5) {
            fade_alpha = 1.0 - fade_alpha;
        }
        sdf_alpha = sdf_alpha * fade_alpha;
    }

    // Erase mode (radius < 0): output pure SDF alpha for destination-out blending.
    // The erase pipeline uses blend: src*0 + dst*(1-src_alpha), so only alpha matters.
    // This clears the target content inside the SDF bounds, preventing sharp
    // unblurred content from bleeding through the subsequent blur draw pass.
    if (radius < 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, sdf_alpha);
    }

    // Convert framebuffer pixel → normalized UV
    let uv = vec2<f32>(
        frag_pos.x / tex_width,
        frag_pos.y / tex_height
    );

    let pixel_size = vec2<f32>(1.0 / tex_width, 1.0 / tex_height);
    
    // Direction vector for separable blur
    let dir = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), direction < 0.5);
    
    // CSS blur() specifies the value as standard deviation (sigma) directly
    // See: https://www.w3.org/TR/filter-effects-1/#funcdef-filter-blur
    // 
    // The W3C spec recommends using three successive box-blurs for sigma >= 2.0:
    // "Three successive box-blurs build a piece-wise quadratic convolution kernel,
    //  which approximates the Gaussian kernel to within roughly 3%."
    //
    // Box size formula from spec: d = floor(sigma * 3 * sqrt(2π) / 4 + 0.5)
    // This simplifies to approximately: d ≈ sigma * 1.8799 ≈ sigma * 1.88
    // 
    // IMPORTANT: d is the TOTAL box width, not the radius!
    // So we sample from -(d-1)/2 to +(d-1)/2, which gives d total samples.
    let sigma = max(radius, 1.0);
    
    // Calculate box size per W3C formula: d = floor(sigma * 3 * sqrt(2π) / 4 + 0.5)
    // sqrt(2π) ≈ 2.5066, so 3 * 2.5066 / 4 ≈ 1.88
    let d = i32(floor(sigma * 1.8799 + 0.5));
    let box_size = max(d, 1);
    
    // Half-width for sampling: sample from -half to +half (total of box_size samples when odd)
    let half = (box_size - 1) / 2;
    
    var color = vec4<f32>(0.0);
    var total_weight: f32 = 0.0;

    // Center sample
    color += textureSample(u_texture, u_sampler, uv);
    total_weight += 1.0;

    // Bilinear-optimized box blur: pair adjacent texels via hardware
    // interpolation.  A fetch at half-offset (i+0.5) returns the average
    // of texels at i and i+1, so multiply by 2 to recover their sum
    // (box weight = 1 each).  This halves texture fetches vs per-texel
    // sampling.
    var i: i32 = 1;
    loop {
        if (i + 1 > half) { break; }
        let tap_offset = f32(i) + 0.5;
        let step = dir * pixel_size * tap_offset;
        color += (textureSample(u_texture, u_sampler, uv + step)
                + textureSample(u_texture, u_sampler, uv - step)) * 2.0;
        total_weight += 4.0;
        i += 2;
    }

    // Unpaired edge sample when per-side count is odd
    if (i <= half) {
        let step = dir * pixel_size * f32(i);
        color += textureSample(u_texture, u_sampler, uv + step)
               + textureSample(u_texture, u_sampler, uv - step);
        total_weight += 2.0;
    }

    let final_color = color / total_weight;

    // Scale the premultiplied blur result by SDF alpha.
    // The erase pass already cleared the target, so premultiplied alpha blending
    // writes the blur result directly: src + dst*(1-src_alpha) = src + 0 = src.
    // Where content was opaque, blur_alpha ≈ 1 → compositor sees opaque blurred content.
    // Where content was transparent, blur_alpha ≈ 0 → compositor blur shows through.
    return final_color * sdf_alpha;
}
