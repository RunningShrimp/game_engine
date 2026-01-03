// Material Editor Benchmarks
//
// Measures material editing and shader compilation performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct MaterialProperty {
    name: String,
    value: PropertyValue,
}

#[derive(Clone, Debug)]
enum PropertyValue {
    Float(f32),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Color([f32; 4]),
    Texture(String),
    Int(i32),
    Bool(bool),
}

#[derive(Clone, Debug)]
struct Material {
    id: String,
    name: String,
    shader: String,
    properties: HashMap<String, MaterialProperty>,
    render_queue: i32,
    double_sided: bool,
}

impl Material {
    fn new(name: String, shader: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            shader,
            properties: HashMap::new(),
            render_queue: 2000,
            double_sided: false,
        }
    }

    fn set_property(&mut self, name: String, value: PropertyValue) {
        self.properties.insert(name, MaterialProperty { name, value });
    }

    fn get_property(&self, name: &str) -> Option<&MaterialProperty> {
        self.properties.get(name)
    }

    fn clone_with_changes(&self, changes: Vec<(String, PropertyValue)>) -> Self {
        let mut cloned = self.clone();
        for (name, value) in changes {
            cloned.set_property(name, value);
        }
        cloned
    }
}

struct MaterialEditor {
    materials: HashMap<String, Material>,
    clipboard: Option<Material>,
}

impl MaterialEditor {
    fn new() -> Self {
        Self {
            materials: HashMap::new(),
            clipboard: None,
        }
    }

    fn create_material(&mut self, material: Material) {
        self.materials.insert(material.id.clone(), material);
    }

    fn update_material(&mut self, id: &str, material: Material) {
        self.materials.insert(id.to_string(), material);
    }

    fn get_material(&self, id: &str) -> Option<&Material> {
        self.materials.get(id)
    }

    fn copy_material(&mut self, id: &str) -> bool {
        if let Some(mat) = self.materials.get(id) {
            self.clipboard = Some(mat.clone());
            true
        } else {
            false
        }
    }

    fn paste_material(&mut self) -> Option<Material> {
        self.clipboard.as_ref().map(|mat| {
            let mut new_mat = mat.clone();
            new_mat.id = uuid::Uuid::new_v4().to_string();
            new_mat.name = format!("{}_copy", mat.name);
            new_mat
        })
    }

    fn duplicate_material(&mut self, id: &str) -> Option<String> {
        self.get_material(id).map(|mat| {
            let mut new_mat = mat.clone();
            let new_id = uuid::Uuid::new_v4().to_string();
            new_mat.id = new_id.clone();
            new_mat.name = format!("{}_duplicate", mat.name);
            self.materials.insert(new_id.clone(), new_mat);
            new_id
        })
    }
}

// Test fixtures
fn create_test_material(count: usize) -> Material {
    let mut mat = Material::new(format!("Material_{}", count), "StandardPBR".to_string());

    mat.set_property("albedo".to_string(), PropertyValue::Color([1.0, 1.0, 1.0, 1.0]));
    mat.set_property("metallic".to_string(), PropertyValue::Float(0.5));
    mat.set_property("roughness".to_string(), PropertyValue::Float(0.5));
    mat.set_property("normal".to_string(), PropertyValue::Texture("normal.png".to_string()));
    mat.set_property("emissive".to_string(), PropertyValue::Vector3([0.0, 0.0, 0.0]));

    mat
}

fn bench_material_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_creation");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter(|| {
                let mut editor = MaterialEditor::new();
                for i in 0..n {
                    let mat = create_test_material(i);
                    editor.create_material(mat);
                }
            });
        });
    }

    group.finish();
}

fn bench_material_property_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_property_update");
    group.measurement_time(Duration::from_secs(10));

    let mut editor = MaterialEditor::new();
    let mat = create_test_material(0);
    let mat_id = mat.id.clone();
    editor.create_material(mat);

    for update_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(update_count),
            update_count,
            |b, &count| {
                b.iter(|| {
                    for i in 0..count {
                        if let Some(mat) = editor.get_material(&mat_id) {
                            let updated = mat.clone_with_changes(vec![
                                ("albedo".to_string(), PropertyValue::Color([
                                    (i as f32 / 100.0),
                                    0.5,
                                    0.5,
                                    1.0,
                                )]),
                            ]);
                            editor.update_material(&mat_id, updated);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_material_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_clone");
    group.measurement_time(Duration::from_secs(10));

    let material = create_test_material(0);

    group.bench_function("clone_with_5_properties", |b| {
        b.iter(|| {
            black_box(material.clone());
        });
    });

    group.finish();
}

fn bench_material_copy_paste(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_copy_paste");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("copy_paste", |b| {
        let mut editor = MaterialEditor::new();
        let mat = create_test_material(0);
        let mat_id = mat.id.clone();
        editor.create_material(mat);

        b.iter(|| {
            black_box(editor.copy_material(&mat_id));
            black_box(editor.paste_material());
        });
    });

    group.finish();
}

fn bench_material_duplicate(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_duplicate");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter(|| {
                let mut editor = MaterialEditor::new();
                let mat = create_test_material(0);
                let mat_id = mat.id.clone();
                editor.create_material(mat);

                for _ in 0..n {
                    editor.duplicate_material(&mat_id);
                }
            });
        });
    }

    group.finish();
}

fn bench_shader_switch(c: &mut Criterion) {
    let mut group = c.benchmark_group("shader_switch");
    group.measurement_time(Duration::from_secs(10));

    let shaders = vec![
        "StandardPBR",
        "Unlit",
        "Toon",
        "NormalDebug",
        "Wireframe",
    ];

    group.bench_function("switch_shaders", |b| {
        let mut editor = MaterialEditor::new();
        let mat = create_test_material(0);
        let mat_id = mat.id.clone();
        editor.create_material(mat);

        b.iter(|| {
            for shader in &shaders {
                if let Some(mat) = editor.get_material(&mat_id) {
                    let mut updated = mat.clone();
                    updated.shader = shader.to_string();
                    editor.update_material(&mat_id, updated);
                }
            }
        });
    });

    group.finish();
}

fn bench_material_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_search");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 500, 1_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            let mut editor = MaterialEditor::new();
            for i in 0..n {
                let mat = create_test_material(i);
                editor.create_material(mat);
            }

            b.iter(|| {
                // Search for materials with specific property
                editor
                    .materials
                    .iter()
                    .filter(|(_, mat)| mat.shader == "StandardPBR")
                    .count();
            });
        });
    }

    group.finish();
}

criterion_group!(
    name = material_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_material_creation,
        bench_material_property_update,
        bench_material_clone,
        bench_material_copy_paste,
        bench_material_duplicate,
        bench_shader_switch,
        bench_material_search
);

criterion_main!(material_benches);
