%Crate shaderc Rust 后面改用这个库去运行时编译shader%
cd..
cd shaders
glslc.exe shader.vert -o shader.vert.spv
glslc.exe shader.frag -o shader.frag.spv
pause