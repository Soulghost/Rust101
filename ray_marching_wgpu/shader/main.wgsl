const PI: f32 = 3.14159265359;
const EPSILON: f32 = 1.19209290e-07;
const F32_MAX: f32 = 3.40282347e+38;
const MAX_ARRAY_SIZE = 255;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct CameraUniform {
    eye_ray: Ray,
    near_far_ssize: vec4<f32>,
    fov_reserved: vec4<f32>
};

struct Shape {
    type_index: i32,
    material_index: i32,
    op_index: i32,
    next_index: i32,
    data: array<f32, 8>
};

struct ShapeUniform {
    shapes: array<Shape>
};

struct PBRMaterial {
    albedo: vec4<f32>, // 0 - 16
    emission: vec4<f32>, // 16 - 32
    metallic: f32, // 32 - 36
    roughness: f32, // 36 - 40
    ao: f32, // 40 - 44
    pad0: f32, // 44 - 48
};

struct PBRMaterialUniform {
    materials: array<PBRMaterial>
};

struct DirectionalLight {
    direction: vec4<f32>, // 0 - 16
    color: vec4<f32> // 16 - 32
};

struct SceneUniform {
    background_color: vec4<f32>,
    main_light: DirectionalLight,
    root_indices: array<i32>
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> u_camera: CameraUniform;

@group(1) @binding(0)
var<storage, read> u_scene: SceneUniform;

@group(1) @binding(1)
var<storage, read> u_shape: ShapeUniform;

@group(1) @binding(2)
var<storage, read> u_material: PBRMaterialUniform;

@group(2) @binding(0)
var t_cloud: texture_2d<f32>;

@group(2) @binding(1)
var s_cloud: sampler; 

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let d = sample_texture_2d_as_3d(t_cloud, s_cloud, 8, 8, vec3<f32>(64.0), vec3<f32>(in.tex_coords.xy, 0.5)).rgb;
    // return vec4(d, 1.0);

    var ray = generate_ray(in.tex_coords);
    return cast_ray(ray);
}

struct Hit {
    valid: f32,
    distance: f32,
    index: i32,
    material_index: i32
}

// Shading
fn cast_ray(_ray: Ray) -> vec4<f32> {
    var ray = _ray;
    var result = vec3<f32>(0.0, 0.0, 0.0);

    // FIXME: fixed lighting
    let background_color = u_scene.background_color.rgb;
    let light_dir = u_scene.main_light.direction.xyz;
    let light_radiance = u_scene.main_light.color.rgb;
    var color_mask = vec3<f32>(1.0);
    var source_metallic = 0.0;
    var hit_dist = 100.0;
    for (var depth = 0; depth < 2; depth++) {
        // ray marching
        let hit = ray_march(ray, 1e5);

        // sample emeission halo
        {
            let max_dist = 1e5;
            let march_accuracy = 1e-3;
            var dist = 0.0;
            var nearest_hit = Hit();
            var sample_point = vec3<f32>();
            nearest_hit.distance = F32_MAX;
            for (var i = 0; i < 300; i++) {
                let p = ray.origin + ray.direction * dist;
                let hit = scene_sdf(p);
                if hit.distance < march_accuracy {
                    break;
                }
                
                dist += hit.distance;
                let shape = u_shape.shapes[hit.index];
                let material = u_material.materials[shape.material_index];
                if hit.distance < nearest_hit.distance && length(material.emission.rgb) > 0.1 {
                    nearest_hit.distance = hit.distance;
                    nearest_hit.index = hit.index;
                    sample_point = p;
                }
                if dist >= max_dist {
                    break;
                }
            }

            var emission_halo = vec3<f32>(0.0);
            if nearest_hit.distance < F32_MAX {
                let a = 0.3;
                let b = 0.0;
                let c = 0.2; // Adjust this for the width of the halo
                let shape = u_shape.shapes[nearest_hit.index];
                let material = get_blend_material(shape, sample_point);
                let emission_halo_atten = a * exp(-0.5 * pow((nearest_hit.distance - b) / c, 2.0));
                let emission_lighting = material.emission.rgb * material.albedo.rgb;
                emission_halo += emission_lighting * emission_halo_atten * color_mask;
            }
            result += emission_halo;
        }

        if hit.valid < 0.5 {
            // hit skybox
            if depth == 0 {
                // return the skybox color
                result += background_color;
            } else {
                result += background_color * color_mask;
            }
            break;
        } 

        let shape = u_shape.shapes[hit.index];
        let p = ray.origin + ray.direction * hit.distance;
        if shape.type_index == 6 {
            // cloud
            // Volumetric Cloud
            result += shading_volumetric_cloud(p, ray.direction, shape, t_cloud, light_dir);
            break;
        }

        // normal shading
        let normal = calculate_normal(hit, p);
        var material = get_blend_material(shape, p);
        if depth == 0 {
            source_metallic = material.metallic;
            hit_dist = hit.distance;
        }

        // directional lighting
        let view = normalize(ray.origin - p);
        let light = -light_dir;
        let direct_lighting = pbr_lighting(p, material, view, normal, light, light_radiance);
        let shadow_atten = calculate_shadow_attenuation(p, normal, light, 16.0);
        result += direct_lighting * shadow_atten * color_mask;

        // sample all emission lights
        {
            var index = u_scene.root_indices[0];
            var i = 0;
            let max_dist = 1e3;
            while index != -1 {
                let shape = u_shape.shapes[index];
                var dist = shape_chain_sdf(shape, p);
                if index != hit.index && dist < max_dist {
                    let material = get_blend_material(shape, p);
                    let emission_lighting_position = vec3<f32>(shape.data[0], shape.data[1], shape.data[2]);
                    if length(material.emission.rgb) > 0.1 {
                        // emission light source
                        let light = normalize(emission_lighting_position - p);
                        let emission_lighting = material.emission.rgb * material.albedo.rgb;
                        let indir_lighting = pbr_lighting(p, material, view, normal, light, emission_lighting);
                        let indir_atten = min(1.0 / (dist * dist), 1.0);
                        result += indir_lighting * indir_atten * color_mask;
                    }
                }
                i += 1;
                index = u_scene.root_indices[i];
            }
        }

        // self emission
        let emission = material.emission.rgb * material.albedo.rgb * color_mask;
        result += emission;

        if length(emission.rgb) > 0.5 {
            // emission object does not have scatters
            break;
        }    

        // the ground reflects nothing
        let is_ground = material.albedo.x < 0.0;
        if is_ground {
            break;
        } 
        
        // new ray
        let reflection_normal_bias = 1e-1;
        let reflection_dir = reflect(-view, normal);
        var reflection_orig = vec3<f32>();
        if dot(reflection_dir, normal) >= 0.0 {
            reflection_orig = p + normal * reflection_normal_bias;
        } else {
            reflection_orig = p - normal * reflection_normal_bias;
        }
        ray.origin = reflection_orig;
        ray.direction = reflection_dir;
        color_mask *= 0.75 * source_metallic;
    }

    // depth debug
    // {
    //     let depth = hit_dist;
    //     return vec4(vec3(hit_dist / 100.0), 1.0);
    // }

    return vec4(post_processing(_ray, hit_dist, result), 1.0);
}

fn post_processing(ray: Ray, dist: f32, input: vec3<f32>) -> vec3<f32> {
    var result = input;

    // global fog (depth)
    // {
    //     let hit_position = ray.origin + ray.direction * dist;
    //     let fog_start = 0.0;
    //     let fog_end = 80.0;
    //     let fog_density = 0.5;
    //     let fog_color = vec3<f32>(1.0);
    //     var fog_factor = (dist - fog_start) / (fog_end - fog_start);
    //     fog_factor = 1.0 + pow(fog_factor - 1.0, 3.0);
    //     fog_factor = saturate(fog_factor * fog_density);
    //     result = lerp3(result, fog_color, fog_factor);
    // }

    return tone_mapping(result);
}

fn generate_ray(frag_coords: vec2<f32>) -> Ray {
    let near = u_camera.near_far_ssize.x;
    let fov = u_camera.fov_reserved.x;
    let size = u_camera.near_far_ssize.zw;
    let aspect = size.x / size.y;

    let ndc = vec2<f32>(-(frag_coords.x - 0.5), -(frag_coords.y - 0.5)) * 2.0;
    
    // Field of view and aspect ratio
    var tan_half_fov = tan(radians(fov * 0.5));
    let aspect_ratio = size.x / size.y;

    // Compute ray direction in camera space
    let ray_dir_cam_space = vec3<f32>(tan_half_fov * aspect_ratio * ndc.x, tan_half_fov * ndc.y, 1.0);

    // Build a camera-to-world transformation matrix
    let w = normalize(u_camera.eye_ray.direction);
    let u = normalize(cross(w, vec3<f32>(0.0, 1.0, 0.0)));
    let v = cross(u, w);
    let cam_to_world = mat3x3<f32>(u, v, w);

    // Transform ray direction to world space
    let ray_dir_world_space = normalize(cam_to_world * ray_dir_cam_space);

    var ray = Ray();
    ray.origin = u_camera.eye_ray.origin;
    ray.direction = ray_dir_world_space;
    return ray;
}

// Volumetric Cloud
fn shading_volumetric_cloud(origin: vec3<f32>, 
                            direction: vec3<f32>, 
                            cloud_cube: Shape,
                            cloud_texture: texture_2d<f32>,
                            light_dir: vec3<f32>) -> vec3<f32> {
    let n_steps = 64;
    let step_size = 0.02;
    let density_scale = 0.1;
    // let offset = vec3<f32>(0.52, 0.5, 0.5);
    let offset = vec3<f32>(0.0);
    let n_light_steps = 16;
    let light_step_size = 0.06;
    let light_absorb = 2.02;
    let darkness_threshold = 0.15;
    
    let cube_origin = vec3<f32>(cloud_cube.data[0], cloud_cube.data[1], cloud_cube.data[2]);
    let cube_extent = vec3<f32>(cloud_cube.data[3], cloud_cube.data[4], cloud_cube.data[5]);
    var ray_origin = (origin - cube_origin) / cube_extent;
    // if n_light_steps > 0 {
    //     return ray_origin;
    // }

    var density = 0.0;
    var light_density = 0.0;
    var final_light = 0.0;
    var transmittance = 0.97;
    var light_step_dir = -light_dir;
    
    for (var i = 0; i < n_steps; i++) {
        ray_origin += direction * step_size;
        let sample_pos = ray_origin + offset;
        let sample_uv = saturate((sample_pos + 1.0) / 2.0);
        let d = sample_texture_2d_as_3d(t_cloud, s_cloud, 8, 8, vec3<f32>(64.0), sample_uv).r; 
        density += d * density_scale;

        // light
        var light_origin = sample_pos;
        for (var j = 0; j < n_light_steps; j++) {
            light_origin += light_step_dir * light_step_size;
            let sample_uv = saturate((light_origin + 1.0) / 2.0);
            let d = sample_texture_2d_as_3d(t_cloud, s_cloud, 8, 8, vec3<f32>(64.0), sample_uv).r; 
            light_density += d;
        }

        let light_transmission = exp(-light_density);
        let shadow = darkness_threshold + light_transmission * (1.0 - darkness_threshold);
        final_light += density * transmittance * shadow;
        transmittance *= exp(-density * light_absorb);
    }

    let transmission = exp(-density);
    let cloud_light_color = vec3<f32>(1.0);
    let cloud_shadow_color = vec3<f32>(0.52, 0.61, 0.68) * 0.6;
    let cloud_color = lerp3(cloud_shadow_color, cloud_light_color, final_light);
    return lerp3(cloud_color, u_scene.background_color.rgb, transmission);
}

fn sample_texture_2d_as_3d(t: texture_2d<f32>, s: sampler, n_rows: i32, n_cols: i32, size: vec3<f32>, uv: vec3<f32>) -> vec4<f32> {
    // test 3d texture
    // Calculate the slice index from the z-coordinate.
    let slice_index = i32(uv.z * size.z); // Assuming z is already in [0,1]

    // Calculate the row and column of the slice in the texture atlas.
    let row = slice_index / n_rows; // 8 slices per row
    let col = slice_index % n_cols; // 8 slices per column

    // Calculate the size of each slice in texture coordinates.
    let slice_size = vec2<f32>(1.0 / (size.x / f32(n_rows)), 1.0 / (size.y / f32(n_cols))); // 8 slices per row and column

    // Calculate the offset within the texture atlas.
    let slice_offset = vec2(f32(col) * slice_size.x, f32(row) * slice_size.y);

    // Calculate the final texture coordinates.
    let atals_coords = slice_offset + vec2<f32>(uv.x * slice_size.x, uv.y * slice_size.y);
    return textureSample(t_cloud, s, atals_coords);
}

// Ray Marching
fn ray_march(ray: Ray, max_dist: f32) -> Hit {
    var result = Hit();
    result.valid = 0.0;

    var dist = 0.0;
    let march_accuracy = 1e-3;
    for (var i = 0; i < 300; i++) {
        let p = ray.origin + ray.direction * dist;
        let hit = scene_sdf(p);
        if hit.distance < march_accuracy {
            result.valid = 1.0;
            result.distance = dist;
            result.index = hit.index;
        }
        dist += hit.distance;
        if dist >= max_dist {
            break;
        }
    }
    return result;
}

fn scene_sdf(p: vec3<f32>) -> Hit {
    var index = u_scene.root_indices[0];
    var hit = Hit();
    hit.distance = F32_MAX;
    var i = 0;
    while index != -1 {
        let shape = u_shape.shapes[index];
        var dist = shape_chain_sdf(shape, p);
        if dist < hit.distance {
            hit.distance = dist;
            hit.index = index;
        }
        i += 1;
        index = u_scene.root_indices[i];
    }
    return hit;
}

fn shape_chain_sdf(shape: Shape, p: vec3<f32>) -> f32 {
    var sdf_f = shape_sdf(shape, p);
    var cur = shape;
    var next_index = shape.next_index;
    while next_index != -1 {
        let next = u_shape.shapes[next_index];
        let sdf_i = shape_sdf(next, p);
        sdf_f = op_sdf(sdf_f, cur.op_index, sdf_i);
        cur = next;
        next_index = next.next_index;
    }
    return sdf_f;
}

fn shape_sdf(shape: Shape, p: vec3<f32>) -> f32 {
    // impl ShapeType {
    //     pub fn to_index(&self) -> i32 {
    //         match self {
    //             ShapeType::Sphere => 0,
    //             ShapeType::Cube => 1,
    //             ShapeType::CubeFrame => 2,
    //             ShapeType::Torus => 3,
    //             ShapeType::DeathStar => 4,
    //             ShapeType::Helix => 5,
    //         }
    //     }
    // }
    var type_index = shape.type_index;
    switch (type_index) {
        case 0: {
            return sphere_sdf(shape, p);
        }
        case 1: {
            return cube_sdf(shape, p);
        }
        case 6: {
            return cube_sdf(shape, p);
        }
        default: {
            return F32_MAX;
        }
    }
}

fn sphere_sdf(sphere: Shape, p: vec3<f32>) -> f32 {
    let center = vec3<f32>(sphere.data[0], sphere.data[1], sphere.data[2]);
    let radius = sphere.data[3];
    return length(center - p) - radius;
}

fn cube_sdf(cube: Shape, p: vec3<f32>) -> f32 {
    let center = vec3<f32>(cube.data[0], cube.data[1], cube.data[2]);
    let most_front_up_right = vec3<f32>(cube.data[3], cube.data[4], cube.data[5]);
    var d_abs = p - center;
    d_abs.x = abs(d_abs.x);
    d_abs.y = abs(d_abs.y);
    d_abs.z = abs(d_abs.z);

    let d = d_abs - most_front_up_right;
    var d_clamped = d;
    d_clamped.x = max(d.x, 0.0);
    d_clamped.y = max(d.y, 0.0);
    d_clamped.z = max(d.z, 0.0);
    return length(d_clamped) + min(max(max(d.x, d.y), d.z), 0.0);
}

fn op_sdf(sdf_a: f32, op: i32, sdf_b: f32) -> f32 {
    // impl ShapeOpType {
    //     pub fn to_index(&self) -> i32 {
    //         match self {
    //             ShapeOpType::Nop => 0,
    //             ShapeOpType::Union => 1,
    //             ShapeOpType::Subtraction => 2,
    //             ShapeOpType::Intersection => 3,
    //             ShapeOpType::SmoothUnion => 4,
    //         }
    //     }
    // }
    switch (op) {
        case 1: {
            return min(sdf_a, sdf_b);
        }
        case 2: {
            return max(sdf_a, -sdf_b);
        }
        case 3: {
            return max(sdf_a, sdf_b);
        }
        case 4: {
            let k = 1.0;
            let h = clamp(0.5 + 0.5 * (sdf_b - sdf_a) / k, 0.0, 1.0);
            return lerp(sdf_b, sdf_a, h) - k * h * (1.0 - h);
        }
        default: {
            return F32_MAX;
        }
    }
}

fn calculate_normal(hit: Hit, p: vec3<f32>) -> vec3<f32> {
    let shape = u_shape.shapes[hit.index];
    var e = vec2(1.0,-1.0)*0.5773*0.0005;
    return normalize( e.xyy*scene_sdf(p + e.xyy ).distance + 
					  e.yyx*scene_sdf(p + e.yyx ).distance + 
					  e.yxy*scene_sdf(p + e.yxy ).distance + 
					  e.xxx*scene_sdf(p + e.xxx ).distance );
}

// PBR
fn pbr_lighting(p: vec3<f32>, material: PBRMaterial, view: vec3<f32>, normal: vec3<f32>, light: vec3<f32>, incident_radiance: vec3<f32>) -> vec3<f32> {
    var albedo = material.albedo.xyz;
    if albedo.x < 0.0 {
        // ground material
        let k = i32(p.x * 0.5 + 1000.0) + i32(p.z * 0.5 + 1000.0);
        if k % 2 != 0 {
            albedo = vec3<f32>(1.0, 1.0, 1.0) * 0.8;
        } else {
            albedo = vec3<f32>(1.0, 1.0, 1.0) * 0.3;
        }
    }
    let ambient = u_scene.background_color.rgb * albedo * (1.0 - material.ao);
    let f0 = lerp3(vec3<f32>(0.04), albedo, material.metallic);
    let half_vec = normalize((view + light) * 0.5);
    let ndf = normal_distribution_ggx(normal, half_vec, material.roughness);
    let g = geometry_simth(normal, view, light, material.roughness);
    let f = fresnel_schlick(max(dot(half_vec, view), 0.0), f0);
    let ks = f;
    let kd = (vec3<f32>(1.0) - ks) * (1.0 - material.metallic);
    let numerator = f * ndf * g;
    let denominator = 4.0 * max(dot(normal, view), 0.0) * max(dot(normal, light), 0.0) + EPSILON;
    let specular = numerator / denominator;
    let diffuse = albedo * kd / PI;
    return ambient + (diffuse + specular) * incident_radiance * max(dot(light, normal), 0.0);
}

fn normal_distribution_ggx(normal: vec3<f32>, half_vec: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(dot(normal, half_vec), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;
    let numerator = a2;
    var denominator = n_dot_h2 * (a2 - 1.0) + 1.0;
    denominator = PI * denominator * denominator;
    return numerator / denominator;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = r * r / 8.0;
    let numerator = n_dot_v;
    let denominator = n_dot_v * (1.0 - k) + k;
    return numerator / denominator;
}

fn geometry_simth(normal: vec3<f32>, view: vec3<f32>, light: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(normal, view), 0.0);
    let n_dot_l = max(dot(normal, light), 0.0);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let f1 = (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta + EPSILON, 5.0);
    return f0 + f1;
}

fn calculate_shadow_attenuation(p: vec3<f32>, normal: vec3<f32>, light: vec3<f32>, k: f32) -> f32 {
    let normal_bias = 1e-1;
    var origin = vec3<f32>();
    if dot(normal, light) >= 0.0 {
        origin = p + normal * normal_bias;
    } else {
        origin = p - normal * normal_bias;
    }

    var ray = Ray();
    ray.origin = origin;
    ray.direction = light;

    let max_dist = 1000.0;
    var dist = 0.0;
    var result = 1.0;
    let march_accuracy = 1e-3;

    for (var i = 0; i < 256; i++) {
        let p = ray.origin + ray.direction * dist;
        let hit = scene_sdf(p);
        if hit.distance < march_accuracy {
            return 0.0;
        }
        result = min(result, k * hit.distance / dist);
        dist += hit.distance;
        if dist >= max_dist {
            break;
        }
    }
    return result;
}

fn get_blend_material(shape: Shape, p: vec3<f32>) -> PBRMaterial {
    var material = PBRMaterial();

    let max_dist = 1.0;
    let dist_bias = 1e-3;
    var weights_total = 0.0;
    var material_indices = array<i32, MAX_ARRAY_SIZE>();
    var material_weights = array<f32, MAX_ARRAY_SIZE>();
    var array_size = 0;
    var dist = shape_sdf(shape, p);
    var cur = shape;
    var next_index = shape.next_index;

    // material 0
    if dist < max_dist {
        material_indices[array_size] = cur.material_index;
        material_weights[array_size] = 1.0 / (dist + dist_bias);
        weights_total += material_weights[array_size];
        array_size += 1;
    }
    
    // other materials
    while next_index != -1 {
        let next = u_shape.shapes[next_index];
        dist = shape_sdf(next, p);

        if dist < max_dist {
            material_indices[array_size] = next.material_index;
            material_weights[array_size] = 1.0 / (dist + dist_bias);
            weights_total += material_weights[array_size];
            array_size += 1;
        }

        cur = next;
        next_index = next.next_index;
    }

    // blend
    for (var i = 0; i < array_size; i++) {
        let m = u_material.materials[material_indices[i]];
        let weight = material_weights[i] / weights_total;
        material.albedo += m.albedo * weight;
        material.emission += m.emission * weight;
        material.metallic += m.metallic * weight;
        material.roughness += m.roughness * weight;
        material.ao += m.ao * weight;
    }
    
    return material;
}

// HDR
fn tone_mapping(_color: vec3<f32>) -> vec3<f32> {
    var color = _color;
    color.x = color.x / (color.x + 1.0);
    color.y = color.y / (color.y + 1.0);
    color.z = color.z / (color.z + 1.0);
    return color;
}

// Math
fn lerp(a: f32, b: f32, s: f32) -> f32 {
    return a + (b - a) * s;
}

fn lerp3(a: vec3<f32>, b: vec3<f32>, s: f32) -> vec3<f32> {
    return a + (b - a) * s;
}

fn lerp4(a: vec4<f32>, b: vec4<f32>, s: f32) -> vec4<f32> {
    return a + (b - a) * s;
}