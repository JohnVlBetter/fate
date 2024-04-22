%Crate shaderc Rust 后面改用这个库去运行时编译shader%
cd..
cd crates/viewer/shaders
glslc.exe model.vert -o model.vert.spv
glslc.exe model.frag -o model.frag.spv
pause