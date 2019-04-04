use serde::{Serialize, Deserialize};
use crate::client::Client;
use crate::Error;
use crate::tag::{Request, Response};
use super::{Device, Dimension, Dimensions};

impl Client {
    pub fn get_device_by_name(&self, name: &str) -> Result<Device, Error> {
        let url = format!("{}/api/internal/device/{}", self.endpoint(), name);

        #[derive(Serialize, Deserialize, Debug)]
        struct Wrapper {
            device: Device,
        }

        Ok(self.get::<Wrapper>(&url)?.device)
    }

    pub fn get_custom_dimensions(&self) -> Result<Dimensions, Error> {
        let url = format!("{}/api/internal/customdimensions", self.endpoint());
        self.get(&url)
    }

    pub fn add_custom_dimension(&self, d: &Dimension) -> Result<Dimension, Error> {
        let url = format!("{}/api/internal/customdimension", self.endpoint());

        #[derive(Serialize, Deserialize, Debug)]
        struct Wrapper {
            #[serde(rename = "customDimension")]
            dimension: Dimension,
        }

        Ok(self.post::<_, Wrapper>(&url, d)?.dimension)
    }

    pub fn update_populators(&self, column: &str, r: &Request) -> Result<Response, Error> {
        let url = format!("{}/api/internal/batch/customdimensions/{}/populators", self.endpoint(), column);
        self.post(&url, r)
    }
}
