


mod erosion_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "src/erosion.glsl",
    }
}