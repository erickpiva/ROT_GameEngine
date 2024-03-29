use crate::rot_primitives::mesh::Mesh;
use crate::rot_primitives::{Camera, Instance, Material, Vertex};
use crate::Renderer;
use std::error::Error;
use std::path::Path;
use wgpu::util::DeviceExt;

use crate::rot_pipeline::{Pipeline, PipelineBuilder};
use log::Level::Info;
use nalgebra as na;

pub struct Object {
    pub name: String,

    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,

    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,

    pub pipelines: Vec<Pipeline>,
}

impl Object {
    pub fn load<P: AsRef<Path>>(renderer: &Renderer, path: P, name: &str) -> Self {
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true).unwrap();

        let containing_folder = path.as_ref().parent().unwrap();

        let mut materials = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;

            materials.push(Material::build(
                containing_folder.join(diffuse_path),
                renderer,
                &mat.name,
            ));
        }

        let pipeline_buider = PipelineBuilder {
            name: format!("{} pipeline", name).as_str(),
            uniform_material: true,
            uniform_camera: true,
            uniform_light: false,
            vertex_shader_path: format!("shaders/{}.vert.spv", name).as_str(),
            fragment_shader_path: format!("shaders/{}.frag.spv", name).as_str(),
            vertex_buffer_layout: vec![Vertex::desc(), Instance::desc()],
        };

        let pipelines = materials
            .iter()
            .map(|material| Pipeline::new(renderer, pipeline_buider))
            .collect::<Vec<_>>();

        let mut meshes: Vec<Mesh> = Vec::new();
        for model in obj_models {
            let mut vertices = Vec::new();
            for i in 0..model.mesh.positions.len() / 3 {
                //It will go through all de groups of (x,y,z), meaning, each vertex
                vertices.push(Vertex {
                    position: [
                        model.mesh.positions[i * 3], // for i=0 -> 0     for i=1 -> 3        for i=2 -> 6
                        model.mesh.positions[i * 3 + 1], //            1                4                   7
                        model.mesh.positions[i * 3 + 2], //            2                5                   8
                    ],
                    tex_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ],
                })
            }

            meshes.push(Mesh::new(
                renderer,
                vertices,
                model.mesh.indices,
                &model.name,
            ))
        }

        let instances = Instance::default();
        let instance_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} Index Buffer", name)),
                    contents: bytemuck::cast_slice(&[instances.uniform.model]),
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                });

        Self {
            name: name.to_string(),

            meshes,
            materials,
            instances: vec![],
            instance_buffer,
            pipelines,
        }
    }

    //potential to paralelize
    fn update_instance(&mut self, renderer: &Renderer) {
        self.instances.iter_mut().map(|instance| instance.update());

        renderer.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(
                &self
                    .instances
                    .iter()
                    .map(|isnst| isnst.uniform.model)
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
        )
    }

    pub fn add_instance(&mut self, renderer: &Renderer, isometries: Vec<na::Isometry3<f32>>) {
        self.instances = isometries
            .iter()
            .map(|&isometry| Instance::new(isometry))
            .collect();

        self.instance_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} Instance Buffer", &self.name)),
                    contents: bytemuck::cast_slice(
                        self.instances
                            .iter()
                            .map(|instance| instance.uniform.model)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                });
    }

    pub fn set_instance(&mut self, renderer: &Renderer, isometries: Vec<na::Isometry3<f32>>) {
        self.instances.clear();

        self.add_instance(renderer, isometries);
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        let render_pass = renderer.get_render_pass();
        render_pass.set_pipeline(&self.pipelines[0].render_pipeline);
        render_pass.set_bind_group(0, &object.materials[0].bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera.unwrap().bind_group, &[]);
        render_pass.set_vertex_buffer(0, object.meshes[0].vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, object.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            object.meshes[0].index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(
            0..object.meshes[0].size as _,
            0,
            0..object.instances.len() as _,
        );
    }

    pub fn on_update(&mut self, renderer: &Renderer) {
        self.update_instance(renderer);
    }
}
