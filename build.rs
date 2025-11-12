fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_descriptors = protox::compile(["ev3_ros_bridge.proto"], ["proto/"]).unwrap();
    tonic_build::configure()
        .build_server(true)
        .compile_fds(file_descriptors)
        .unwrap();
    Ok(())
}
