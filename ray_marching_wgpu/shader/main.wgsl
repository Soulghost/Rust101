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

struct SphereShape /*: Shape*/ {
    type_index: i32, // 0
    material_index: i32, // 4
    center: vec3<f32>, // 16
    radius: f32, // 32
    pad0: array<f32, 3>
};

struct SceneUniform {
    root_index: i32,
    shape_count: i32,
    pad0: array<f32, 2>,
    shapes: array<Shape>
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
    // return vec4(f32(scene.shapes[0].material_index), 0.0, 0.0, 1.0);

    var ray = generate_ray(in.tex_coords);
    let hit = ray_march(ray, 1e5);
    return lerp4(vec4(1.0, 0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), hit.valid);
}

struct Hit {
    valid: f32,
    distance: f32,
    index: i32
}

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
            result.index = 0;
        }
        dist += hit.distance;
        if dist >= max_dist {
            break;
        }
    }

    return result;
}

fn sdf(p: vec3<f32>) -> Hit {
    // mock the sphere
    let root_index = scene.root_index;
    let sphere = scene.shapes[root_index];
    let center = vec3<f32>(sphere.data[0], sphere.data[1], sphere.data[2]);
    let radius = sphere.data[3];

    var hit = Hit();  
    hit.valid = 1.0;
    hit.distance = sphere_sdf(p, center, radius);
    hit.index = 0;
    return hit;
}

fn sphere_sdf(p: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(center - p) - radius;
}

fn generate_ray(frag_coords: vec2<f32>) -> Ray {
    let near = camera.near_far_ssize.x;
    let fov = camera.fov_reserved.x;
    let size = camera.near_far_ssize.zw;
    let aspect = size.x / size.y;

    let ndc = vec2<f32>((frag_coords.x - 0.5), -(frag_coords.y - 0.5)) * 2.0;
    
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

fn lerp4(a: vec4<f32>, b: vec4<f32>, s: f32) -> vec4<f32> {
    return a + (b - a) * s;
}