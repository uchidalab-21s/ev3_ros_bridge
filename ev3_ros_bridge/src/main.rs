use std::sync::Arc;
use tokio::sync::Mutex;

use safe_drive::msg::common_interfaces::geometry_msgs;
use safe_drive::{context::Context, error::DynError, topic::{publisher::Publisher, subscriber::Subscriber}};
use ev3_ros_msg::msg::Sensor;

use tonic::{transport::Server, Request, Response, Status};

// use ev3_ros_tonic::pb;
use ev3_proto::{ev3_ros_bridge_server::Ev3RosBridge, CmdVel, SensorData, WriteResponse};

pub struct Ev3RosBridgeServer {
    subscriber: Arc<Mutex<Subscriber<geometry_msgs::msg::Twist>>>,
    publisher: Publisher<Sensor>,
}

impl Ev3RosBridgeServer {
    fn new(subscriber: Arc<Mutex<Subscriber<geometry_msgs::msg::Twist>>>, publisher: Publisher<Sensor>) -> Ev3RosBridgeServer {
        Ev3RosBridgeServer { subscriber, publisher }
    }
}

#[tonic::async_trait]
impl Ev3RosBridge for Ev3RosBridgeServer {
    async fn read_cmd_vel(&self, _: Request<()>) -> Result<Response<CmdVel>, Status> {
        let mut subscriber = self.subscriber.lock().await;
        let cmd_vel = subscriber.recv().await.unwrap();
        Ok(Response::new(CmdVel {
            x: cmd_vel.linear.x,
            y: cmd_vel.linear.y,
            theta: cmd_vel.angular.z,
        }))
    }

    async fn write_sensor_data(
        &self,
        request: Request<SensorData>,
    ) -> Result<Response<WriteResponse>, Status> {
        let metadata_len = request.metadata().len() as i32;
        let _sensor_data = request.into_inner();

        self.publisher.send(&Sensor {
            touch_pressed_state: _sensor_data.touch_pressed_state,
            gyro_angle: _sensor_data.gyro_angle,
            gyro_rotational_speed: _sensor_data.gyro_rotational_speed,
            ultrasonic_distance: _sensor_data.ultrasonic_distance,
        }).unwrap();
        Ok(Response::new(WriteResponse { size: metadata_len }))
    }
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let ctx = Context::new()?;
    let node = ctx.create_node("ev3_ros_bridge", None, Default::default())?;

    let subscriber = node.create_subscriber::<geometry_msgs::msg::Twist>("/cmd_vel", None)?;
    let publisher = node.create_publisher::<Sensor>("/sensor_data", None)?;

    let server = Ev3RosBridgeServer::new(Arc::new(Mutex::new(subscriber)), publisher);

    let args: Vec<String> = std::env::args().collect();
    const DEFAULT_ADDRESS: &str = "127.0.0.1:50051";
    let socket_address = match args.get(1) {
        Some(str) => str.parse().unwrap(), // コマンドライン引数にヘンな値が入ってたら落とす
        None => DEFAULT_ADDRESS.parse().unwrap(), // プログラマの責任で安全にunwrapできる
    };

    println!("Server listening on {}", socket_address);

    Server::builder()
        .add_service(ev3_proto::ev3_ros_bridge_server::Ev3RosBridgeServer::new(server))
        .serve(socket_address)
        .await
        .unwrap();
    Ok(())
}
