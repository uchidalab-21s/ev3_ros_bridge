fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/ev3_ros_bridge.proto")?;
    Ok(())
}
