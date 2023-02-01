#version 450 core

/** GLOBAL FUNCTIONS **/
vec3 unit(vec3 v)
{
    return v / length(v);
}

float length_squared(vec3 v)
{
  return (v.x * v.x) + (v.y * v.y) + (v.z * v.z);
}

bool is_near_zero(vec3 v)
{
    return abs(v.x) < 1e-8 &&
            abs(v.y) < 1e-8 &&
            abs(v.z) < 1e-8;
}

float reflectance(float cosine, float ref_idx)
{
  float r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
  r0 = r0 * r0;
  return r0 + (1.0 - r0) * pow(1.0 - cosine, 5.0);
}

/** STRUCTURES **/
struct Camera{
  vec3 origin;
  vec3 lower_left_corner;
  vec3 horizontal;
  vec3 vertical;
  vec3 up;
  vec3 u;
  vec3 v;
  vec3 w;
  float lens_radius;
};

struct Ray {
    vec3 origin;
    vec3 dir;
};

struct HitRecord {
  vec3 normal;
  vec3 point;
  float t;

  bool front_face;

  uint mat_type;
  vec3 albedo;
  float fuzz_or_ir;
};

struct ScatterResult {
  bool scattered;
  vec3 attenuation;
  Ray ray;
};

struct Sphere {
    float radius;
    uint mat_type;
    float fuzz_or_ir;

    vec3 albedo;
    vec3 center;
};

/** SHADER LAYOUT **/
layout(local_size_x = 1 , local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) writeonly buffer Data {
    uint colors[];
} data;

layout(set = 0, binding = 1) readonly buffer Config {
  uint num_spheres;
  uint sample_count;
  uint max_bounces;
  uint width;
  uint height;

  Camera camera;
} config;

layout(set = 0, binding = 2) readonly buffer Scene {
  Sphere spheres[];
} scene;

layout(push_constant) uniform PushConstantData {
  uint index;
} push_constants;

/** RANDOM NUMBER GENERATOR **/
// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
uint hash( uint x ) {
    x += ( x << 10u );
    x ^= ( x >>  6u );
    x += ( x <<  3u );
    x ^= ( x >> 11u );
    x += ( x << 15u );
    return x;
}

// Compound versions of the hashing algorithm I whipped together.
uint hash( uvec2 v ) { return hash( v.x ^ hash(v.y)                         ); }
uint hash( uvec3 v ) { return hash( v.x ^ hash(v.y) ^ hash(v.z)             ); }
uint hash( uvec4 v ) { return hash( v.x ^ hash(v.y) ^ hash(v.z) ^ hash(v.w) ); }

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
float floatConstruct( uint m ) {
    const uint ieeeMantissa = 0x007FFFFFu; // binary32 mantissa bitmask
    const uint ieeeOne      = 0x3F800000u; // 1.0 in IEEE binary32

    m &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
    m |= ieeeOne;                          // Add fractional part to 1.0

    float  f = uintBitsToFloat( m );       // Range [1:2]
    return f - 1.0;                        // Range [0:1]
}

// Pseudo-random value in half-open range [0:1].
float random( float x ) { return floatConstruct(hash(floatBitsToUint(x))); }
float random( vec2  v ) { return floatConstruct(hash(floatBitsToUint(v))); }
float random( vec3  v ) { return floatConstruct(hash(floatBitsToUint(v))); }
float random( vec4  v ) { return floatConstruct(hash(floatBitsToUint(v))); }

vec3 randomDiskPoint(vec3 rand) {
  float x = 1.0;
  while (true)
  {
    vec3 p = vec3(
        -1.0 + (random(rand + vec3(x)) * 2.0),
        -1.0 + (random(-rand - vec3(x)) * 2.0),
        0.0
    );

    if(length_squared(p) < 1.0)
      return p;

    x++;
  }
}

vec3 randomSpherePoint(vec3 rand) {
  float x = 1.0;
  vec3 p;

  while(true)
  {
    p = vec3(
        -1.0 + (random(rand + vec3(x)) * 2.0),
        -1.0 + (random(-rand - vec3(x)) * 2.0),
        -1.0 + (random(rand / (2.0 * x)) * 2.0)
     );

    if(length_squared(p) < 1.0)
        return p;
    x++;
  }
}
/** SCATTER FUNCS **/
ScatterResult scatter_lambertian(Ray ray, HitRecord hit_record, vec3 rand)
{
  Ray scatter_ray;
  scatter_ray.origin = hit_record.point;
  scatter_ray.dir = hit_record.normal + unit(randomSpherePoint(rand));

  if(is_near_zero(scatter_ray.dir)){
    scatter_ray.dir = hit_record.normal;
  }

  ScatterResult result;
  result.scattered = true;
  result.attenuation = hit_record.albedo;
  result.ray = scatter_ray;
  return result;
}

ScatterResult scatter_metal(Ray ray, HitRecord hit_record, vec3 rand)
{
    Ray scatter_ray;
    scatter_ray.origin = hit_record.point;
    scatter_ray.dir = reflect(unit(ray.dir), hit_record.normal) 
    + (hit_record.fuzz_or_ir * randomSpherePoint(rand * 69.0));

    ScatterResult result;
    result.scattered = dot(scatter_ray.dir, hit_record.normal) > 0.0;
    result.attenuation = hit_record.albedo;
    result.ray = scatter_ray;

    return result;
}

ScatterResult scatter_dielectric(Ray ray, HitRecord hit_record, vec3 rand)
{
  Ray scatter_ray;
  scatter_ray.origin = hit_record.point;

  vec3 unit_dir = unit(ray.dir);

  float refraction_ratio = hit_record.front_face ?  
    1.0 / hit_record.fuzz_or_ir :
    hit_record.fuzz_or_ir;

  float cos_theta = min(dot(-unit_dir, hit_record.normal), 1.0);
  float sin_theta = sqrt(1.0 - (cos_theta * cos_theta));

  scatter_ray.dir = refraction_ratio * sin_theta > 1.0 || 
                    reflectance(cos_theta, refraction_ratio) > random(rand * cos_theta) ?
                    reflect(unit_dir, hit_record.normal) :
                    refract(unit_dir, hit_record.normal, refraction_ratio);
    
  ScatterResult result;
  result.scattered = true;
  result.attenuation = vec3(1.0, 1.0, 1.0);
  result.ray = scatter_ray;

  return result;
}

/** RAY PROCESSING **/
vec3 ProcessRay(Ray ray, uint x, uint y, uint z)
{
  vec3 out_color = vec3(1.0);

  for(uint b = 0;; b++)
  {
    if(b == config.max_bounces - 1) {
      return vec3(0.0, 0.0, 0.0);
    }

    bool hit_anything = false;
    HitRecord hit_record;

    float t_max = 1.0 / 0.0; // INF
    for(uint i = 0; i < config.num_spheres; i++)
    {
      Sphere sphere = scene.spheres[i];

      vec3 oc = ray.origin - sphere.center;
      float a = length_squared(ray.dir);
      float half_b = dot(oc, ray.dir);
      float c = length_squared(oc) - (sphere.radius * sphere.radius);

      float discriminant = (half_b * half_b) - (a * c);

      if(discriminant < 0.0) {
        continue;
      }

      float sqrtd = sqrt(discriminant);
      float root = (-half_b - sqrtd) / a;

      if(root <  0.001 || root > t_max){
          root = (-half_b + sqrtd) / a;
        if(root <  0.001 || root > t_max){
          continue;
        }
      }

      t_max = root;
      hit_anything = true;
      
      hit_record.mat_type = sphere.mat_type;
      hit_record.albedo = sphere.albedo;
      hit_record.fuzz_or_ir = sphere.fuzz_or_ir;

      hit_record.t = root;
      hit_record.point = ray.origin + root * ray.dir;

      vec3 outward_normal = (hit_record.point - sphere.center) / sphere.radius;
      hit_record.front_face = dot(ray.dir, outward_normal) < 0.0;
      hit_record.normal = hit_record.front_face ? outward_normal : -outward_normal;
    }

    if(hit_anything)
    {
      ScatterResult scatter;

      switch(hit_record.mat_type)
      {
        case 0:
        {
          scatter = scatter_lambertian(ray, hit_record, vec3(x + b, y + z, hit_record.t));
          break;
        }
        case 1:
        {
          scatter = scatter_metal(ray, hit_record, vec3(x + b, y + z, hit_record.t));
          break;
        }
        case 2:
        {
          scatter = scatter_dielectric(ray, hit_record, vec3(x + b, y + z, hit_record.t));
          break;
        }
      }

      if(scatter.scattered)
      {
        ray = scatter.ray;
        out_color *=  scatter.attenuation;
      }
      else
      {
        return vec3(0.0, 0.0, 0.0);
      }
    }
    else
    {
      float unit_y =  unit(ray.dir).y;
      out_color *= mix(vec3(1.0, 1.0, 1.0), vec3(0.5, 0.7, 0.9), 0.5 * (unit_y + 1.0));
      break;
    }
  }

  return out_color;
}

void main() 
{
  uint index = push_constants.index + gl_GlobalInvocationID.x;
  uint idx = index % config.width;
  uint idy = index / config.width;

  Camera camera = config.camera;
  vec3 color = vec3(0.0);

  for(uint i = 0; i < config.sample_count; i++)
  {
    float u = float(idx) / (config.width - 1.0);
    float v = float(idy) / (config.height - 1.0);

    vec3 rd = camera.lens_radius * randomDiskPoint(vec3(u, v, i));
    vec3 offset = (camera.u * rd.x) + (camera.v * rd.y);

    Ray ray;
    ray.origin = camera.origin + offset;
    ray.dir = (camera.lower_left_corner) 
      + (u * camera.horizontal)
      + (v * camera.vertical) 
      - (camera.origin)
      - (offset);

    color += ProcessRay(ray, idx, idy, i);
    
  }
  float scale = 1.0 / config.sample_count;
  color.x = 256.0 * (clamp(sqrt(color.x * scale), 0.0, 0.999));
  color.y = 256.0 * (clamp(sqrt(color.y * scale), 0.0, 0.999));
  color.z = 256.0 * (clamp(sqrt(color.z * scale), 0.0, 0.999));
  
  data.colors[(index * 3) + 0] = uint(color.x);
  data.colors[(index * 3) + 1] = uint(color.y);
  data.colors[(index * 3) + 2] = uint(color.z);
}


