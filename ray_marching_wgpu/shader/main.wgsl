const PI: f32 = 3.14159265359;
const EPSILON: f32 = 1.19209290e-07;

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
    data: array<f32, 10>
};

// struct SphereShape /*: Shape*/ {
//     type_index: i32, // 0
//     material_index: i32, // 4
//     center: array<f32, 3>, // 8 - 20
//     radius: f32, // 20 - 24
//     ...
// };

struct SceneUniform {
    root_index: i32,
    shape_count: i32,
    pad0: array<f32, 2>,
    shapes: array<Shape>
};

struct PBRMaterial {
    albedo: vec3<f32>, // 0
    emission: vec3<f32>, // 16
    metallic: f32, // 32
    roughness: f32, // 36
    ao: f32, // 40
    pad0: f32, // 44 - 48
};

struct PBRMaterialUniform {
    material_count: i32,
    pad0: array<f32, 3>,
    materials: array<PBRMaterial>
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
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<storage> scene: SceneUniform;

@group(1) @binding(1)
var<storage> material: PBRMaterialUniform;

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
    // return vec4(material.materials[0].albedo, 1.0);
    // return vec4(f32(scene.shapes[0].material_index), 0.0, 0.0, 1.0);

    var ray = generate_ray(in.tex_coords);
    return cast_ray(ray);
}

struct Hit {
    valid: f32,
    distance: f32,
    index: i32
}

// Shading
fn cast_ray(_ray: Ray) -> vec4<f32> {
    var ray = _ray;
    var result = vec3<f32>(0.0, 0.0, 0.0);

    // FIXME: fixed lighting
    let background_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let light_intensity = 10.0;
    let light_dir = vec3<f32>(0.32, -0.77, 0.56);
    let light_radiance = vec3<f32>(1.0, 1.0, 1.0) * light_intensity;
    for (var depth = 0; depth < 1; depth++) {
        // ray marching
        let hit = ray_march(_ray, 1e5);
        if hit.valid < 0.5 {
            if depth == 0 {
                return background_color;
            } else {
                break;
            }
        }

        // shading
        let p = ray.origin + ray.direction * hit.distance;
        let normal = calculate_normal(hit, p);
        let shape = scene.shapes[hit.index];
        let material = material.materials[shape.material_index];
        let view = normalize(ray.origin - p);
        let light = -light_dir;
        let direct_lighting = pbr_lighting(material, view, normal, light, light_radiance);

        // shadow
        let shadow_atten = calculate_shadow_attenuation(p, normal, light);
        result += direct_lighting * shadow_atten;

        // new ray
        let reflection_normal_bias = 1e-3;
        let reflection_dir = reflect(-view, normal);
        var reflection_orig = vec3<f32>();
        if dot(reflection_dir, normal) >= 0.0 {
            reflection_orig = p + normal * reflection_normal_bias;
        } else {
            reflection_orig = p - normal * reflection_normal_bias;
        }
        ray.origin = reflection_orig;
        ray.direction = reflection_dir;
    }
    return vec4(tone_mapping(result), 1.0);
}

fn generate_ray(frag_coords: vec2<f32>) -> Ray {
    let near = camera.near_far_ssize.x;
    let fov = camera.fov_reserved.x;
    let size = camera.near_far_ssize.zw;
    let aspect = size.x / size.y;

    let ndc = vec2<f32>(-(frag_coords.x - 0.5), -(frag_coords.y - 0.5)) * 2.0;
    
    // Field of view and aspect ratio
    var tan_half_fov = tan(radians(fov * 0.5));
    let aspect_ratio = size.x / size.y;

    // Compute ray direction in camera space
    let ray_dir_cam_space = vec3<f32>(tan_half_fov * aspect_ratio * ndc.x, tan_half_fov * ndc.y, 1.0);

    // Build a camera-to-world transformation matrix
    let w = normalize(camera.eye_ray.direction);
    let u = normalize(cross(w, vec3<f32>(0.0, 1.0, 0.0)));
    let v = cross(u, w);
    let cam_to_world = mat3x3<f32>(u, v, w);

    // Transform ray direction to world space
    let ray_dir_world_space = normalize(cam_to_world * ray_dir_cam_space);

    var ray = Ray();
    ray.origin = camera.eye_ray.origin;
    ray.direction = ray_dir_world_space;
    return ray;
}

// Ray Marching
fn ray_march(ray: Ray, max_dist: f32) -> Hit {
    var result = Hit();
    result.valid = 0.0;

    var dist = 0.0;
    let march_accuracy = 1e-3;
    for (var i = 0; i < 300; i++) {
        let p = ray.origin + ray.direction * dist;
        let hit = sdf(p);
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

fn sdf(p: vec3<f32>) -> Hit {
    // FIXME: mock the sphere
    let root_index = scene.root_index;
    let shape = scene.shapes[root_index];

    var hit = Hit();  
    hit.distance = shape_sdf(shape, p);
    hit.index = root_index;
    return hit;
}

fn shape_sdf(shape: Shape, p: vec3<f32>) -> f32 {
    let sphere = shape;
    let center = vec3<f32>(sphere.data[0], sphere.data[1], sphere.data[2]);
    let radius = sphere.data[3];
    return sphere_sdf(p, center, radius);
}

fn sphere_sdf(p: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(center - p) - radius;
}

fn calculate_normal(hit: Hit, p: vec3<f32>) -> vec3<f32> {
    let shape = scene.shapes[hit.index];
    
    let eps_grad = 1e-3;
    let p_x_p = p + vec3<f32>(eps_grad, 0.0, 0.0);
    let p_x_m = p - vec3<f32>(eps_grad, 0.0, 0.0);
    let p_y_p = p + vec3<f32>(0.0, eps_grad, 0.0);
    let p_y_m = p - vec3<f32>(0.0, eps_grad, 0.0);
    let p_z_p = p + vec3<f32>(0.0, 0.0, eps_grad);
    let p_z_m = p - vec3<f32>(0.0, 0.0, eps_grad);

    let sdf_x_p = shape_sdf(shape, p_x_p);
    let sdf_x_m = shape_sdf(shape, p_x_m);
    let sdf_y_p = shape_sdf(shape, p_y_p);
    let sdf_y_m = shape_sdf(shape, p_y_m);
    let sdf_z_p = shape_sdf(shape, p_z_p);
    let sdf_z_m = shape_sdf(shape, p_z_m);
    let normal = vec3<f32>(sdf_x_p - sdf_x_m, sdf_y_p - sdf_y_m, sdf_z_p - sdf_z_m) / (2.0 * eps_grad);
    return normal;
}

// PBR
fn pbr_lighting(material: PBRMaterial, view: vec3<f32>, normal: vec3<f32>, light: vec3<f32>, incident_radiance: vec3<f32>) -> vec3<f32> {
    let albedo = material.albedo;
    let ambient = vec3<f32>(0.03) * albedo * (1.0 - material.ao);
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

fn calculate_shadow_attenuation(p: vec3<f32>, normal: vec3<f32>, light: vec3<f32>) -> f32 {
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
    let hit = ray_march(ray, 1e4);
    return 1.0 - hit.valid;
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