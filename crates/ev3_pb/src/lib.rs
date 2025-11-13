use std::cell::RefCell;
use tonic::Request;
use pb::{CmdVel, SensorData, WriteResponse};

pub struct TonicHandle {
    tokio_runtime: tokio::runtime::Runtime,
    client: RefCell<pb::ev3_ros_bridge_client::Ev3RosBridgeClient<tonic::transport::Channel>>,
}

impl TonicHandle {
    pub fn new(url: &str) -> Self {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        let client = tokio_runtime.block_on(async {
            pb::ev3_ros_bridge_client::Ev3RosBridgeClient::connect(url.to_string())
                .await
                .unwrap()
        });
        Self {
            tokio_runtime: tokio_runtime,
            client: std::cell::RefCell::new(client),
        }
    }

    pub fn read_cmd_vel(&self) -> Result<CmdVel, tonic::Status> {
        let request = Request::new(());
        self.tokio_runtime.block_on(async {
            let mut client = self.client.borrow_mut();
            let response = client.read_cmd_vel(request).await?;
            Ok(response.into_inner())
        })
    }

    pub fn write_sensor_data(&self, sensor_data: SensorData) -> Result<WriteResponse, tonic::Status> {
        let request = Request::new(sensor_data);
        self.tokio_runtime.block_on(async {
            let mut client = self.client.borrow_mut();
            let response = client.write_sensor_data(request).await?;
            Ok(response.into_inner())
        })
    }
}