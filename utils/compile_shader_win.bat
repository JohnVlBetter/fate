%Crate shaderc Rust 后面改用这个库去运行时编译shader%
cd..
cd crates/fate_renderer/shaders
glslc.exe model.vert -o model.vert.spv
glslc.exe model.frag -o model.frag.spv
glslc.exe shadowcaster.vert -o shadowcaster.vert.spv
glslc.exe shadowcaster.frag -o shadowcaster.frag.spv
glslc.exe final.frag -o final.frag.spv
pause