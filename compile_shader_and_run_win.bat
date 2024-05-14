cd crates/fate_renderer/shaders
glslc.exe model.vert -o model.vert.spv
glslc.exe model.frag -o model.frag.spv
glslc.exe shadowcaster.vert -o shadowcaster.vert.spv
glslc.exe shadowcaster.frag -o shadowcaster.frag.spv
glslc.exe final.frag -o final.frag.spv
glslc.exe ssao.frag -o ssao.frag.spv
cd ../../..
cargo run