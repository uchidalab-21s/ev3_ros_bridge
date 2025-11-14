use std::cell::RefCell;
use tonic::Request;
use ev3_proto::{CmdVel, SensorData, WriteResponse};

pub type Result<T> = std::result::Result<T, Ev3PbError>;

#[derive(Debug)]
pub enum Ev3PbError {
    Transport,
    NotFound,
    Permission,
    Unavailable,
    Internal(String),
    Other(tonic::Code, String),
}

impl std::fmt::Display for Ev3PbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ev3PbError::Transport => write!(f, "Transport error"),
            Ev3PbError::NotFound => write!(f, "Not found"),
            Ev3PbError::Permission => write!(f, "Permission denied"),
            Ev3PbError::Unavailable => write!(f, "Service unavailable"),
            Ev3PbError::Internal(msg) => write!(f, "Internal error: {}", msg),
            Ev3PbError::Other(code, msg) => write!(f, "gRPC error {:?}: {}", code, msg),
        }
    }
}

impl std::error::Error for Ev3PbError {}

impl From<tonic::Status> for Ev3PbError {
    fn from(s: tonic::Status) -> Self {
        use tonic::Code::*;
        match s.code() {
            NotFound => Ev3PbError::NotFound,
            PermissionDenied => Ev3PbError::Permission,
            Unavailable => Ev3PbError::Unavailable,
            Internal => Ev3PbError::Internal(s.message().to_string()),
            _ => Ev3PbError::Other(s.code(), s.message().to_string()),
        }
    }
}

pub struct TonicHandle {
    tokio_runtime: tokio::runtime::Runtime,
    client: RefCell<ev3_proto::ev3_ros_bridge_client::Ev3RosBridgeClient<tonic::transport::Channel>>,
}

impl TonicHandle {
    pub fn new(url: &str) -> Self {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        let client = tokio_runtime.block_on(async {
            ev3_proto::ev3_ros_bridge_client::Ev3RosBridgeClient::connect(url.to_string())
                .await
                .unwrap()
        });
        Self {
            tokio_runtime: tokio_runtime,
            client: std::cell::RefCell::new(client),
        }
    }

    pub fn read_cmd_vel(&self) -> Result<CmdVel> {
        let request = Request::new(());
        self.tokio_runtime.block_on(async {
            let mut client = self.client.borrow_mut();
            let response = client.read_cmd_vel(request).await?;
            Ok(response.into_inner())
        })
    }

    pub fn write_sensor_data(&self, sensor_data: SensorData) -> Result<WriteResponse> {
        let request = Request::new(sensor_data);
        self.tokio_runtime.block_on(async {
            let mut client = self.client.borrow_mut();
            let response = client.write_sensor_data(request).await?;
            Ok(response.into_inner())
        })
    }
}