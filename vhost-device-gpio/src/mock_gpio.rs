// Mock GPIO backend device for testing
//
// Copyright 2023 Linaro Ltd. All Rights Reserved.
//          Viresh Kumar <viresh.kumar@linaro.org>
//
// SPDX-License-Identifier: Apache-2.0 or BSD-3-Clause

use log::info;
use std::sync::RwLock;

use crate::gpio::{Error, GpioDevice, GpioState, Result};
use crate::virtio_gpio::*;

use jsonrpc::Client;
use jsonrpc::simple_http::{self, SimpleHttpTransport};
use serde_json::json;
use serde_json::value::to_raw_value;

fn client() -> std::result::Result<Client,simple_http::Error> {
    let url = "http://127.0.0.1:8001";
    let t = SimpleHttpTransport::builder()
        .url(url)?
        .build();

    Ok(Client::with_transport(t))
}


#[derive(Debug)]
pub(crate) struct MockGpioDevice {
    ngpio: u16,
    pub(crate) gpio_names: Vec<String>,
    state: RwLock<Vec<GpioState>>,
    pub num_gpios_result: Result<u16>,
    pub gpio_name_result: Result<String>,
    pub direction_result: Result<u8>,
    set_direction_result: Result<()>,
    value_result: Result<u8>,
    set_value_result: Result<()>,
    set_irq_type_result: Result<()>,
    pub(crate) wait_for_irq_result: Result<bool>,
    pub rpc_client: Client,
}

impl MockGpioDevice {
    pub(crate) fn new(ngpio: u16) -> Self {
        let rpc_client = client().unwrap();
        let mut request = rpc_client.build_request("num_gpios", None);
        let mut response = rpc_client.send_request(request).expect("send_request failed");
        //self.ngpio = (*response.result.unwrap()).from_str();
        println!("{:?}",response);
        let mut resp2 : serde_json::Value = serde_json::from_str((*response.result.unwrap()).get()).unwrap(); 
        println!("{:?}",resp2);
        let ngpio2 : u16 = resp2[1].as_u64().unwrap().try_into().unwrap() ;
        let mut gpio_names = Vec::<String>::new();
        for i in 0..ngpio2 {
            let mut param = json!([i]);
            let mut raw_value = Some(to_raw_value(&param).unwrap());
            let mut request = rpc_client.build_request("gpio_name", raw_value.as_deref());
            let mut response = rpc_client.send_request(request).expect("send_request failed");
            println!("{:?}",response);
            let mut resp2 : serde_json::Value = serde_json::from_str((*response.result.unwrap()).get()).unwrap(); 
            println!("{:?}",resp2);
	    let mut name : String = resp2[1].as_str().unwrap().into(); 
            gpio_names.push(name);
        }

        Self {
            ngpio: ngpio2,
            gpio_names,
            state: RwLock::new(vec![
                GpioState {
                    dir: VIRTIO_GPIO_DIRECTION_NONE,
                    val: None,
                    irq_type: VIRTIO_GPIO_IRQ_TYPE_NONE,
                };
                ngpio2.into()
            ]),
            num_gpios_result: Ok(0),
            gpio_name_result: Ok("".to_string()),
            direction_result: Ok(0),
            set_direction_result: Ok(()),
            value_result: Ok(0),
            set_value_result: Ok(()),
            set_irq_type_result: Ok(()),
            wait_for_irq_result: Ok(true),
            rpc_client,
        }
    }
}

impl GpioDevice for MockGpioDevice {
    fn open(ngpios: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(MockGpioDevice::new(ngpios.try_into().unwrap()))
    }

    fn num_gpios(&self) -> Result<u16> {
        if self.num_gpios_result.is_err() {
            return self.num_gpios_result;
        }
        Ok(self.ngpio)
    }

    fn gpio_name(&self, gpio: u16) -> Result<String> {
        assert!((gpio as usize) < self.gpio_names.len());

        if self.gpio_name_result.is_err() {
            return self.gpio_name_result.clone();
        }

        Ok(self.gpio_names[gpio as usize].clone())
    }

    fn direction(&self, gpio: u16) -> Result<u8> {
        if self.direction_result.is_err() {
            return self.direction_result;
        }
        {
          let param = json!([11,{"rr":"akuku","tr":[34,21]}]);
	  let raw_value = Some(to_raw_value(&param).unwrap());
          let request = self.rpc_client.build_request("test", raw_value.as_deref());
          println!("{:?}",request);
          let response = self.rpc_client.send_request(request).expect("send_request failed");
          println!("{:?}",response)
        }
        Ok(self.state.read().unwrap()[gpio as usize].dir)
    }

    fn set_direction(&self, gpio: u16, dir: u8, value: u32) -> Result<()> {
        info!(
            "gpio {} set direction to {}",
            self.gpio_names[gpio as usize], dir
        );

        if self.set_direction_result.is_err() {
            return self.set_direction_result;
        }

        self.state.write().unwrap()[gpio as usize].dir = dir;
        self.state.write().unwrap()[gpio as usize].val = match dir {
            VIRTIO_GPIO_DIRECTION_NONE => None,
            VIRTIO_GPIO_DIRECTION_IN => self.state.read().unwrap()[gpio as usize].val,
            VIRTIO_GPIO_DIRECTION_OUT => Some(value as u16),

            _ => return Err(Error::GpioDirectionInvalid(dir as u32)),
        };

        Ok(())
    }

    fn value(&self, gpio: u16) -> Result<u8> {
        if self.value_result.is_err() {
            return self.value_result;
        }

        if let Some(val) = self.state.read().unwrap()[gpio as usize].val {
            Ok(val as u8)
        } else {
            Err(Error::GpioCurrentValueInvalid)
        }
    }

    fn set_value(&self, gpio: u16, value: u32) -> Result<()> {
        info!(
            "gpio {} set value to {}",
            self.gpio_names[gpio as usize], value
        );

        if self.set_value_result.is_err() {
            return self.set_value_result;
        }

        self.state.write().unwrap()[gpio as usize].val = Some(value as u16);
        Ok(())
    }

    fn set_irq_type(&self, gpio: u16, value: u16) -> Result<()> {
        info!(
            "gpio {} set irq type to {}",
            self.gpio_name(gpio).unwrap(),
            value
        );
        if self.set_irq_type_result.is_err() {
            return self.set_irq_type_result;
        }

        Ok(())
    }

    fn wait_for_interrupt(&self, _gpio: u16) -> Result<bool> {
        if self.wait_for_irq_result.is_err() {
            return self.wait_for_irq_result;
        }

        Ok(true)
    }
}
